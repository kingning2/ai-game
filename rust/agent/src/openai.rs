use serde::Deserialize;

use crate::event::{AgentEvent, StopReason, Usage};
use crate::message::{ChatMessage, ChatRequest, Role, ToolChoice, ToolDefinition};
use crate::provider::{ChatStreamFuture, ProviderSettings, http_client};
use crate::stream::{ChatStream, FrameDecoder, StreamState, ensure_success};
use crate::{AgentError, AgentResult};

/// OpenAI 兼容协议（`POST {base_url}/chat/completions`，SSE 流式）。
///
/// 覆盖所有 OpenAI 风格网关：OpenAI 官方、豆包方舟兼容模式、DeepSeek、Kimi、OpenRouter 等。
/// 推理增量兼容两种字段名：DeepSeek 系 `reasoning_content` 优先，OpenRouter 系 `reasoning` 兜底
#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    client: reqwest::Client,
    settings: ProviderSettings,
    max_tokens: u32,
}

impl OpenAiProvider {
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
            "{}/chat/completions",
            self.settings.base_url.trim_end_matches('/')
        )
    }
}

/// 统一消息 → OpenAI `messages` 数组。
/// 多轮工具往返：assistant 带 `tool_calls`；结果用 `role:"tool"` + `tool_call_id`
pub(crate) fn to_protocol_messages(messages: &[ChatMessage]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .map(|message| {
            let mut object =
                serde_json::json!({ "role": message.role, "content": message.content });
            if !message.tool_calls.is_empty() {
                object["tool_calls"] = serde_json::json!(
                    message
                        .tool_calls
                        .iter()
                        .map(|call| serde_json::json!({
                            "id": call.id,
                            "type": "function",
                            "function": { "name": call.name, "arguments": call.arguments },
                        }))
                        .collect::<Vec<_>>()
                );
                // 部分 OpenAI 兼容网关拒绝 tool_calls 消息带 content 字段（省略比空串更稳）
                if message.content.is_empty() {
                    object["content"] = serde_json::Value::Null;
                }
            }
            if message.role == Role::Tool {
                // 本层每次只装一条结果（[ChatMessage::tool_result] 的约定），取第一个
                if let Some(result) = message.tool_results.first() {
                    object["tool_call_id"] = serde_json::json!(result.call_id);
                    object["content"] = serde_json::json!(result.content);
                }
            }
            object
        })
        .collect()
}

/// 统一工具定义 → OpenAI `tools` 数组（`{"type":"function","function":{…}}`）
pub(crate) fn to_protocol_tools(tools: &[ToolDefinition]) -> Vec<serde_json::Value> {
    tools
        .iter()
        .map(|tool| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters,
                },
            })
        })
        .collect()
}

impl crate::provider::AgentProvider for OpenAiProvider {
    fn id(&self) -> &'static str {
        "openai"
    }

    fn chat_stream<'a>(&'a self, request: ChatRequest) -> ChatStreamFuture<'a> {
        Box::pin(async move {
            request.validate()?;
            self.settings.require_key()?;
            let ChatRequest {
                model,
                messages,
                thinking,
                tools,
                tool_choice,
            } = request;
            let model = model.unwrap_or_else(|| self.settings.model.clone());

            // 方舟统一思考参数（GLM / DeepSeek / Doubao 系均支持 enabled/disabled；
            // 部分模型不支持 disabled 时由网关报 400，本层透传意图不吞错误）
            let mut body = serde_json::json!({
                "model": model,
                "messages": to_protocol_messages(&messages),
                "stream": true,
                "max_tokens": self.max_tokens,
                // 请求最后一个 chunk 附带用量（OpenAI 2024+ 规范，主流网关都支持）
                "stream_options": { "include_usage": true },
            });
            if let Some(mode) = thinking {
                body["thinking"] = serde_json::json!({ "type": mode });
            }
            if !tools.is_empty() {
                body["tools"] = serde_json::json!(to_protocol_tools(&tools));
                body["tool_choice"] = serde_json::json!(match tool_choice {
                    ToolChoice::Auto => "auto",
                    ToolChoice::Required => "required",
                });
            }

            let response = self
                .client
                .post(self.endpoint())
                .bearer_auth(&self.settings.api_key)
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

// ===== 上游 SSE → 统一事件 =====

/// 一帧 data 的 JSON（只取需要的字段，未知字段忽略以保持网关兼容）
#[derive(Debug, Deserialize)]
struct OpenAiChunk {
    #[serde(default)]
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
    error: Option<OpenAiErrorBody>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    #[serde(default)]
    delta: OpenAiDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct OpenAiDelta {
    content: Option<String>,
    /// DeepSeek 系推理字段
    reasoning_content: Option<String>,
    /// OpenRouter 系推理字段
    reasoning: Option<String>,
    /// 流式工具调用分片（乱序到达，按 index 聚合）
    #[serde(default)]
    tool_calls: Vec<OpenAiToolCallDelta>,
}

#[derive(Debug, Deserialize)]
struct OpenAiToolCallDelta {
    index: usize,
    id: Option<String>,
    function: Option<OpenAiToolFunctionDelta>,
}

#[derive(Debug, Deserialize)]
struct OpenAiToolFunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u64,
    completion_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorBody {
    message: Option<String>,
}

/// 流式解码：每帧最多产出一个增量事件；结束原因记在 state，`[DONE]` 帧统一发 Done
pub(crate) fn decode_frame(state: &mut StreamState, data: &str) -> AgentResult<Vec<AgentEvent>> {
    let data = data.trim();
    if data.is_empty() {
        return Ok(Vec::new());
    }
    if data == "[DONE]" {
        let stop_reason = state
            .stop_reason
            .as_deref()
            .map(StopReason::from_protocol)
            .unwrap_or(StopReason::EndTurn);
        // 工具调用先于 Done 发出（消费方在 Done 前收齐完整调用列表）
        let mut events = Vec::new();
        if matches!(stop_reason, StopReason::ToolCalls) {
            events.push(AgentEvent::ToolCalls {
                calls: state.take_tool_calls(),
            });
        }
        events.push(AgentEvent::Done { stop_reason });
        return finish(state, events);
    }

    let chunk: OpenAiChunk = serde_json::from_str(data)
        .map_err(|error| AgentError::Protocol(format!("openai chunk 解析失败：{error}")))?;

    if let Some(error) = chunk.error {
        return Err(AgentError::UpstreamEvent(
            error.message.unwrap_or_else(|| "未知错误".to_string()),
        ));
    }

    let mut events = Vec::new();
    if let Some(usage) = chunk.usage {
        events.push(AgentEvent::Usage {
            usage: Usage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
            },
        });
    }
    for choice in chunk.choices {
        let delta = choice.delta;
        // 推理增量：DeepSeek 系字段优先，OpenRouter 系兜底
        let thinking = delta.reasoning_content.or(delta.reasoning);
        if let Some(delta) = thinking.filter(|delta| !delta.is_empty()) {
            events.push(AgentEvent::ThinkingDelta { delta });
        }
        if let Some(delta) = delta.content.filter(|delta| !delta.is_empty()) {
            events.push(AgentEvent::TextDelta { delta });
        }
        // 工具调用分片：id/name/arguments 都是增量，按 index 拼到对应槽
        for fragment in delta.tool_calls {
            let slot = state.tool_slot(fragment.index);
            if let Some(id) = fragment.id {
                slot.id.push_str(&id);
            }
            if let Some(function) = fragment.function {
                if let Some(name) = function.name {
                    slot.name.push_str(&name);
                }
                if let Some(arguments) = function.arguments {
                    slot.arguments.push_str(&arguments);
                }
            }
        }
        if let Some(reason) = choice.finish_reason {
            state.stop_reason = Some(reason);
        }
    }
    Ok(events)
}

/// 已发过 Done 就不再重复（部分网关会多发结束帧）
fn finish(state: &mut StreamState, events: Vec<AgentEvent>) -> AgentResult<Vec<AgentEvent>> {
    if state.done {
        return Ok(Vec::new());
    }
    state.done = true;
    Ok(events)
}

#[cfg(test)]
#[path = "openai_tests.rs"]
mod tests;
