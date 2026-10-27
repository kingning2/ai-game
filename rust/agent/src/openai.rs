use serde::Deserialize;
use utils::FallibleExt;

use crate::event::{AgentEvent, StopReason, Usage};
use crate::message::ChatRequest;
use crate::provider::{ChatStreamFuture, ProviderSettings};
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
            "{}/chat/completions",
            self.settings.base_url.trim_end_matches('/')
        )
    }
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
            } = request;
            let model = model.unwrap_or_else(|| self.settings.model.clone());

            // 方舟统一思考参数（GLM / DeepSeek / Doubao 系均支持 enabled/disabled；
            // 部分模型不支持 disabled 时由网关报 400，本层透传意图不吞错误）
            let mut body = serde_json::json!({
                "model": model,
                "messages": messages,
                "stream": true,
                "max_tokens": self.max_tokens,
                // 请求最后一个 chunk 附带用量（OpenAI 2024+ 规范，主流网关都支持）
                "stream_options": { "include_usage": true },
            });
            if let Some(mode) = thinking {
                body["thinking"] = serde_json::json!({ "type": mode });
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
        return finish(state, vec![AgentEvent::Done { stop_reason }]);
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
mod tests {
    use super::*;

    fn decode(state: &mut StreamState, data: &str) -> Vec<AgentEvent> {
        decode_frame(state, data).unwrap()
    }

    #[test]
    fn text_delta() {
        let mut state = StreamState::default();
        let events = decode(
            &mut state,
            r#"{"id":"1","choices":[{"index":0,"delta":{"content":"你好"},"finish_reason":null}]}"#,
        );
        assert_eq!(
            events,
            [AgentEvent::TextDelta {
                delta: "你好".into()
            }]
        );
    }

    /// DeepSeek reasoning_content 优先，OpenRouter reasoning 兜底
    #[test]
    fn thinking_delta_both_field_names() {
        let mut state = StreamState::default();
        let events = decode(
            &mut state,
            r#"{"choices":[{"delta":{"reasoning_content":"想一下"}}]}"#,
        );
        assert_eq!(
            events,
            [AgentEvent::ThinkingDelta {
                delta: "想一下".into()
            }]
        );

        let events = decode(
            &mut state,
            r#"{"choices":[{"delta":{"reasoning":"再想想"}}]}"#,
        );
        assert_eq!(
            events,
            [AgentEvent::ThinkingDelta {
                delta: "再想想".into()
            }]
        );
    }

    /// 空串增量不发事件（部分网关会发 content:"" 占位帧）
    #[test]
    fn empty_delta_skipped() {
        let mut state = StreamState::default();
        let events = decode(&mut state, r#"{"choices":[{"delta":{"content":""}}]}"#);
        assert!(events.is_empty());
    }

    /// finish_reason 记在 state，[DONE] 时合成 Done
    #[test]
    fn done_uses_last_finish_reason() {
        let mut state = StreamState::default();
        decode(
            &mut state,
            r#"{"choices":[{"delta":{"content":"尾"},"finish_reason":"length"}]}"#,
        );
        let events = decode(&mut state, "[DONE]");
        assert_eq!(
            events,
            [AgentEvent::Done {
                stop_reason: StopReason::MaxTokens
            }]
        );

        // 重复结束帧被吞
        assert!(decode(&mut state, "[DONE]").is_empty());
    }

    /// include_usage 的用量帧：choices 为空数组 + usage 全量
    #[test]
    fn usage_chunk() {
        let mut state = StreamState::default();
        let events = decode(
            &mut state,
            r#"{"choices":[],"usage":{"prompt_tokens":6,"completion_tokens":9}}"#,
        );
        assert_eq!(
            events,
            [AgentEvent::Usage {
                usage: Usage {
                    input_tokens: 6,
                    output_tokens: 9
                }
            }]
        );
    }

    /// 流中错误帧 → Err（HTTP 已 200，只能走 Err 分支）
    #[test]
    fn in_stream_error_is_err() {
        let mut state = StreamState::default();
        let err = decode_frame(
            &mut state,
            r#"{"error":{"message":"模型不存在","type":"invalid_request_error"}}"#,
        )
        .unwrap_err();
        assert!(matches!(err, AgentError::UpstreamEvent(message) if message == "模型不存在"));
    }

    /// 未知 JSON → Protocol 错误（可定位是哪段内容）
    #[test]
    fn malformed_frame_is_protocol_error() {
        let mut state = StreamState::default();
        let err = decode_frame(&mut state, "{not json").unwrap_err();
        assert!(matches!(err, AgentError::Protocol(_)));
        assert!(err.to_string().contains("openai chunk"));
    }
}
