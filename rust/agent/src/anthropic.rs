use serde::Deserialize;

use crate::event::{AgentEvent, StopReason, Usage};
use crate::message::{ChatMessage, ChatRequest, Role, ToolChoice};
use crate::provider::{ChatStreamFuture, ProviderSettings, http_client};
use crate::stream::{ChatStream, FrameDecoder, StreamState, ensure_success};
use crate::{AgentError, AgentResult};

/// Anthropic Messages 协议（`POST {base_url}/v1/messages`，SSE 流式）。
///
/// 与 OpenAI 协议的差异集中在这里消化：
/// - system 不在 messages 里，单独放顶层 `system` 字段
/// - `max_tokens` 必填（取 provider 配置）
/// - messages 要求 role 严格交替，相邻同 role 自动合并（\n\n 连接）
/// - 用量拆在两帧：`message_start` 给 input，`message_delta` 给 output
#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    client: reqwest::Client,
    settings: ProviderSettings,
    max_tokens: u32,
}

impl AnthropicProvider {
    /// `Err` 仅在 reqwest Client 构建失败时出现（TLS 后端初始化等环境问题）
    pub fn new(settings: ProviderSettings, max_tokens: u32) -> AgentResult<Self> {
        Ok(Self {
            client: http_client()?,
            settings,
            max_tokens,
        })
    }

    fn endpoint(&self) -> String {
        format!(
            "{}/v1/messages",
            self.settings.base_url.trim_end_matches('/')
        )
    }
}

impl crate::provider::AgentProvider for AnthropicProvider {
    fn id(&self) -> &'static str {
        "anthropic"
    }

    fn chat_stream<'a>(&'a self, request: ChatRequest) -> ChatStreamFuture<'a> {
        Box::pin(async move {
            request.validate()?;
            self.settings.require_key()?;
            let ChatRequest {
                model,
                messages,
                tools,
                tool_choice,
                ..
            } = request;
            let model = model.unwrap_or_else(|| self.settings.model.clone());
            let (system, turns) = split_messages(&messages);

            let mut body = serde_json::json!({
                "model": model,
                "max_tokens": self.max_tokens,
                "stream": true,
                "messages": turns,
            });
            if let Some(system) = system {
                body["system"] = serde_json::Value::String(system);
            }
            if !tools.is_empty() {
                // Anthropic 工具定义：parameters 改名 input_schema，无外层包装
                body["tools"] = serde_json::json!(
                    tools
                        .iter()
                        .map(|tool| serde_json::json!({
                            "name": tool.name,
                            "description": tool.description,
                            "input_schema": tool.parameters,
                        }))
                        .collect::<Vec<_>>()
                );
                body["tool_choice"] = serde_json::json!(match tool_choice {
                    ToolChoice::Auto => serde_json::json!({ "type": "auto" }),
                    ToolChoice::Required => serde_json::json!({ "type": "any" }),
                });
            }

            let response = self
                .client
                .post(self.endpoint())
                .header("x-api-key", &self.settings.api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send()
                .await
                .map_err(|err| AgentError::Request(err.into()))?;

            ensure_success(response)
                .await
                .map(|response| ChatStream::new(response, decode_frame as FrameDecoder))
        })
    }
}

/// system 拆出（多条拼接）+ 相邻同 role 合并 + 工具消息转 blocks。
///
/// 多轮工具往返的协议形态：
/// - assistant 带 `tool_calls` → content 变 blocks：文本 block + `tool_use` blocks
/// - `Role::Tool` 结果 → `tool_result` block 追加到最近一个 user turn（协议硬性要求）
/// - 其余相邻同 role 合并（Anthropic 要求严格交替）
fn split_messages(messages: &[ChatMessage]) -> (Option<String>, Vec<serde_json::Value>) {
    let mut system_parts: Vec<&str> = Vec::new();
    // (role, content blocks)；文本消息就是一个 text block
    let mut turns: Vec<(Role, Vec<serde_json::Value>)> = Vec::new();

    fn push_text(blocks: &mut Vec<serde_json::Value>, text: &str) {
        if !text.is_empty() {
            blocks.push(serde_json::json!({ "type": "text", "text": text }));
        }
    }

    for message in messages {
        match message.role {
            Role::System => system_parts.push(&message.content),
            Role::Tool => {
                // tool_result 必须挂在 user turn 里；本层每次只装一条结果，取第一个
                if let Some(result) = message.tool_results.first() {
                    let block = serde_json::json!({
                        "type": "tool_result",
                        "tool_use_id": result.call_id,
                        "content": result.content,
                    });
                    match turns.last_mut() {
                        // 已是 user turn（含上一步刚开出的）→ 追加
                        Some((Role::User, blocks)) => blocks.push(block),
                        // 没有 user turn（协议要求结果前必有 assistant 调用，正常不会走到）
                        // → 开一个新 user turn 保证不丢结果
                        _ => turns.push((Role::User, vec![block])),
                    }
                }
            }
            Role::User | Role::Assistant => {
                let mut blocks = Vec::new();
                push_text(&mut blocks, &message.content);
                for call in &message.tool_calls {
                    // arguments 是 JSON 文本；解析失败时塞 _raw 保信息不丢
                    let input = serde_json::from_str::<serde_json::Value>(&call.arguments)
                        .unwrap_or_else(|_| serde_json::json!({ "_raw": call.arguments }));
                    blocks.push(serde_json::json!({
                        "type": "tool_use",
                        "id": call.id,
                        "name": call.name,
                        "input": input,
                    }));
                }
                match turns.last_mut() {
                    Some((role, existing)) if *role == message.role => existing.extend(blocks),
                    _ => turns.push((message.role, blocks)),
                }
            }
        }
    }
    let system = if system_parts.is_empty() {
        None
    } else {
        Some(system_parts.join("\n\n"))
    };
    let wire = turns
        .into_iter()
        .map(|(role, blocks)| serde_json::json!({ "role": role, "content": blocks }))
        .collect();
    (system, wire)
}

// ===== 上游 SSE → 统一事件 =====

/// 一帧 data 的 JSON。两家的帧都靠顶层 `type` 字段路由（比 event: 行可靠）；
/// 裸错误帧（如网关透传的 {"error":{...}}）没有 type，故 kind 可缺省
#[derive(Debug, Deserialize)]
struct AnthropicFrame {
    #[serde(rename = "type", default)]
    kind: Option<String>,

    /// message_start：{ message: { usage: { input_tokens } } }
    message: Option<AnthropicMessageStart>,
    /// content_block_start 的 content_block 载荷（tool_use 时带 id/name）
    #[serde(default)]
    index: Option<usize>,
    content_block: Option<AnthropicContentBlock>,
    /// content_block_delta / message_delta 共用的 delta 载荷
    delta: Option<AnthropicDelta>,
    /// message_delta：{ usage: { output_tokens } }
    usage: Option<AnthropicUsageOut>,
    /// error 帧
    error: Option<AnthropicErrorBody>,
}

#[derive(Debug, Deserialize)]
struct AnthropicMessageStart {
    #[serde(default)]
    usage: AnthropicUsageIn,
}

/// content_block_start 的载荷：只关心 tool_use（text/thinking 块无增量状态要记）
#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    kind: String,
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct AnthropicUsageIn {
    #[serde(default)]
    input_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsageOut {
    #[serde(default)]
    output_tokens: u64,
}

#[derive(Debug, Default, Deserialize)]
struct AnthropicDelta {
    /// text_delta 的文本
    text: Option<String>,
    /// thinking_delta 的思考文本
    thinking: Option<String>,
    /// input_json_delta 的工具参数 JSON 增量
    partial_json: Option<String>,
    /// message_delta 的结束原因
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorBody {
    message: Option<String>,
}

pub(crate) fn decode_frame(state: &mut StreamState, data: &str) -> AgentResult<Vec<AgentEvent>> {
    let data = data.trim();
    if data.is_empty() {
        return Ok(Vec::new());
    }
    // 兼容网关可能混入的 OpenAI 风格结束哨兵
    if data == "[DONE]" {
        return finish(state, StopReason::EndTurn);
    }

    let frame: AnthropicFrame = serde_json::from_str(data)
        .map_err(|error| AgentError::Protocol(format!("anthropic frame 解析失败：{error}")))?;

    match frame.kind.as_deref() {
        // 兼容 OpenAI 风格的流中错误：{"error": {...}}（无 type 时也兜底到这里）
        _ if frame.error.is_some() => Err(AgentError::UpstreamEvent(
            frame
                .error
                .and_then(|error| error.message)
                .unwrap_or_else(|| "未知错误".to_string()),
        )),
        Some("message_start") => {
            if let Some(message) = frame.message {
                state.input_tokens = Some(message.usage.input_tokens);
            }
            Ok(Vec::new())
        }
        Some("content_block_start") => {
            if let (Some(index), Some(block)) = (frame.index, frame.content_block)
                && block.kind == "tool_use"
            {
                let slot = state.tool_slot(index);
                slot.id = block.id.unwrap_or_default();
                slot.name = block.name.unwrap_or_default();
            }
            Ok(Vec::new())
        }
        Some("content_block_delta") => {
            let mut events = Vec::new();
            if let Some(delta) = frame.delta {
                if let Some(thinking) = delta.thinking.filter(|text| !text.is_empty()) {
                    events.push(AgentEvent::ThinkingDelta { delta: thinking });
                }
                if let Some(text) = delta.text.filter(|text| !text.is_empty()) {
                    events.push(AgentEvent::TextDelta { delta: text });
                }
                if let Some(partial) = delta.partial_json {
                    // input_json_delta：参数 JSON 拼到本 block 对应的槽；缺 index 时退回末槽
                    let index = frame
                        .index
                        .unwrap_or_else(|| state.tool_calls.len().saturating_sub(1));
                    state.tool_slot(index).arguments.push_str(&partial);
                }
            }
            Ok(events)
        }
        // 收齐 output → 发 Usage；stop_reason 记下，等 message_stop 合成 Done
        Some("message_delta") => {
            let mut events = Vec::new();
            if let Some(usage) = frame.usage {
                events.push(AgentEvent::Usage {
                    usage: Usage {
                        input_tokens: state.input_tokens.unwrap_or(0),
                        output_tokens: usage.output_tokens,
                    },
                });
            }
            if let Some(stop_reason) = frame.delta.and_then(|delta| delta.stop_reason) {
                state.stop_reason = Some(stop_reason);
            }
            Ok(events)
        }
        Some("message_stop") => finish(state, StopReason::EndTurn),
        // ping / content_block_start / content_block_stop / 未知类型：忽略（前向兼容）
        _ => Ok(Vec::new()),
    }
}

fn finish(state: &mut StreamState, default_reason: StopReason) -> AgentResult<Vec<AgentEvent>> {
    if state.done {
        return Ok(Vec::new());
    }
    state.done = true;
    let stop_reason = state
        .stop_reason
        .as_deref()
        .map(StopReason::from_protocol)
        .unwrap_or(default_reason);
    // 工具调用先于 Done 发出（消费方在 Done 前收齐完整调用列表）
    let mut events = Vec::new();
    if matches!(stop_reason, StopReason::ToolCalls) {
        events.push(AgentEvent::ToolCalls {
            calls: state.take_tool_calls(),
        });
    }
    events.push(AgentEvent::Done { stop_reason });
    Ok(events)
}

#[cfg(test)]
#[path = "anthropic_tests.rs"]
mod tests;
