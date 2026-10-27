use serde::Deserialize;
use utils::FallibleExt;

use crate::event::{AgentEvent, StopReason, Usage};
use crate::message::{ChatMessage, ChatRequest, Role};
use crate::provider::{ChatStreamFuture, ProviderSettings};
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
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .wrap_err(|message| AgentError::Config(format!("HTTP 客户端构建失败：{message}")))?;
        Ok(Self {
            client,
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
                model, messages, ..
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

/// system 拆出（多条拼接）+ 相邻同 role 合并（Anthropic 要求严格交替）
fn split_messages(messages: &[ChatMessage]) -> (Option<String>, Vec<ChatMessage>) {
    let mut system_parts: Vec<&str> = Vec::new();
    let mut turns: Vec<ChatMessage> = Vec::new();
    for message in messages {
        match message.role {
            Role::System => system_parts.push(&message.content),
            Role::User | Role::Assistant => match turns.last_mut() {
                Some(last) if last.role == message.role => {
                    last.content.push_str("\n\n");
                    last.content.push_str(&message.content);
                }
                _ => turns.push(message.clone()),
            },
        }
    }
    let system = if system_parts.is_empty() {
        None
    } else {
        Some(system_parts.join("\n\n"))
    };
    (system, turns)
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
        Some("content_block_delta") => {
            let mut events = Vec::new();
            if let Some(delta) = frame.delta {
                if let Some(thinking) = delta.thinking.filter(|text| !text.is_empty()) {
                    events.push(AgentEvent::ThinkingDelta { delta: thinking });
                }
                if let Some(text) = delta.text.filter(|text| !text.is_empty()) {
                    events.push(AgentEvent::TextDelta { delta: text });
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
    Ok(vec![AgentEvent::Done { stop_reason }])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(state: &mut StreamState, data: &str) -> Vec<AgentEvent> {
        decode_frame(state, data).unwrap()
    }

    #[test]
    fn full_stream_synthesis() {
        let mut state = StreamState::default();
        // message_start 只记 input，不产事件
        assert!(decode(
            &mut state,
            r#"{"type":"message_start","message":{"id":"msg_1","usage":{"input_tokens":25,"output_tokens":0}}}"#
        ).is_empty());

        assert_eq!(
            decode(
                &mut state,
                r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"你好"}}"#
            ),
            [AgentEvent::TextDelta {
                delta: "你好".into()
            }]
        );
        assert_eq!(
            decode(
                &mut state,
                r#"{"type":"content_block_delta","index":1,"delta":{"type":"thinking_delta","thinking":"推理中"}}"#
            ),
            [AgentEvent::ThinkingDelta {
                delta: "推理中".into()
            }]
        );

        // message_delta 合成 Usage（input 来自 message_start 的记忆）
        assert_eq!(
            decode(
                &mut state,
                r#"{"type":"message_delta","delta":{"stop_reason":"max_tokens","stop_sequence":null},"usage":{"output_tokens":15}}"#
            ),
            [AgentEvent::Usage {
                usage: Usage {
                    input_tokens: 25,
                    output_tokens: 15
                }
            }]
        );

        // message_stop 合成 Done，用上一步记下的 stop_reason
        assert_eq!(
            decode(&mut state, r#"{"type":"message_stop"}"#),
            [AgentEvent::Done {
                stop_reason: StopReason::MaxTokens
            }]
        );
    }

    #[test]
    fn ping_and_unknown_types_ignored() {
        let mut state = StreamState::default();
        assert!(decode(&mut state, r#"{"type":"ping"}"#).is_empty());
        assert!(
            decode(
                &mut state,
                r#"{"type":"content_block_start","index":0,"content_block":{"type":"text"}}"#
            )
            .is_empty()
        );
        assert!(decode(&mut state, r#"{"type":"brand_new_frame_v99"}"#).is_empty());
    }

    #[test]
    fn error_frame_is_err() {
        let mut state = StreamState::default();
        let err = decode_frame(
            &mut state,
            r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#,
        )
        .unwrap_err();
        assert!(matches!(err, AgentError::UpstreamEvent(message) if message == "Overloaded"));
    }

    /// 裸 error JSON（无 type 字段）也能兜住
    #[test]
    fn bare_error_object_is_err() {
        let mut state = StreamState::default();
        let err = decode_frame(&mut state, r#"{"error":{"message":"boom"}}"#).unwrap_err();
        assert!(matches!(err, AgentError::UpstreamEvent(message) if message == "boom"));
    }

    #[test]
    fn malformed_frame_is_protocol_error() {
        let mut state = StreamState::default();
        let err = decode_frame(&mut state, "{{{").unwrap_err();
        assert!(matches!(err, AgentError::Protocol(_)));
        assert!(err.to_string().contains("anthropic frame"));
    }

    // ===== split_messages =====

    #[test]
    fn system_extracted_and_adjacent_roles_merged() {
        let messages = vec![
            ChatMessage::system("规则一"),
            ChatMessage::system("规则二"),
            ChatMessage::user("第一问"),
            ChatMessage::user("补充"),
            ChatMessage::assistant("回答"),
            ChatMessage::user("第二问"),
        ];
        let (system, turns) = split_messages(&messages);
        assert_eq!(system.as_deref(), Some("规则一\n\n规则二"));
        assert_eq!(turns.len(), 3);
        assert_eq!(turns[0].content, "第一问\n\n补充");
        assert_eq!(turns[1].role, Role::Assistant);
        assert_eq!(turns[2].content, "第二问");
    }

    #[test]
    fn no_system_gives_none() {
        let messages = vec![ChatMessage::user("hi")];
        let (system, turns) = split_messages(&messages);
        assert_eq!(system, None);
        assert_eq!(turns.len(), 1);
    }
}
