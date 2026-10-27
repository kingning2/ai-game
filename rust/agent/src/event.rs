use serde::{Deserialize, Serialize};

/// 令牌用量（对齐前端 Usage 组件的字段）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// 流结束原因。两家协议的原生取值统一收敛到 [`StopReason::from_protocol`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// 正常说完（openai "stop" / anthropic "end_turn"）
    EndTurn,
    /// 达到 max_tokens 被截断（openai "length" / anthropic "max_tokens"）
    MaxTokens,
    /// 其他（tool_calls / stop_sequence 等本版本未支持的语义）
    Other,
}

impl StopReason {
    /// 协议原生字符串 → 统一语义（未知取值归 Other，不报错——前向兼容新 finish 原因）
    pub fn from_protocol(raw: &str) -> Self {
        match raw {
            "stop" | "end_turn" | "stop_sequence" => Self::EndTurn,
            "length" | "max_tokens" => Self::MaxTokens,
            _ => Self::Other,
        }
    }
}

/// 统一事件流：两种协议的 SSE 帧都归一到这里，直接喂前端 Message / Thinking / Usage 组件。
///
/// 错误不进事件——流中错误走 `ChatStream::next_event` 的 `Err` 分支
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    /// assistant 文本增量（拼接后即完整回复）
    TextDelta { delta: String },

    /// 思考/推理增量（对齐 Thinking 组件；不支持推理的模型不会出现）
    ThinkingDelta { delta: String },

    /// 令牌用量（流结束前发出；input 或 output 任一收齐即发，缺的一侧为 0）
    Usage { usage: Usage },

    /// 流正常结束
    Done { stop_reason: StopReason },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// serde tag 约定：server 的 SSE 转发直接 to_string，字段名变更要过这里
    #[test]
    fn event_serde_wire_format() {
        let text = serde_json::to_string(&AgentEvent::TextDelta {
            delta: "你好".into(),
        })
        .unwrap();
        assert_eq!(text, r#"{"type":"text_delta","delta":"你好"}"#);

        let done = serde_json::to_string(&AgentEvent::Done {
            stop_reason: StopReason::EndTurn,
        })
        .unwrap();
        assert_eq!(done, r#"{"type":"done","stop_reason":"end_turn"}"#);

        let usage = serde_json::to_string(&AgentEvent::Usage {
            usage: Usage {
                input_tokens: 6,
                output_tokens: 9,
            },
        })
        .unwrap();
        assert_eq!(
            usage,
            r#"{"type":"usage","usage":{"input_tokens":6,"output_tokens":9}}"#
        );
    }

    #[test]
    fn stop_reason_mapping() {
        assert_eq!(StopReason::from_protocol("stop"), StopReason::EndTurn);
        assert_eq!(StopReason::from_protocol("end_turn"), StopReason::EndTurn);
        assert_eq!(StopReason::from_protocol("length"), StopReason::MaxTokens);
        assert_eq!(
            StopReason::from_protocol("max_tokens"),
            StopReason::MaxTokens
        );
        assert_eq!(StopReason::from_protocol("tool_calls"), StopReason::Other);
    }
}
