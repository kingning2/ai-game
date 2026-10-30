use serde::{Deserialize, Serialize};

use crate::message::ToolCall;

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
    /// 模型要求调用工具（openai "tool_calls" / anthropic "tool_use"）
    ToolCalls,
    /// 其他（stop_sequence 等本版本未支持的语义）
    Other,
}

impl StopReason {
    /// 协议原生字符串 → 统一语义（未知取值归 Other，不报错——前向兼容新 finish 原因）
    pub fn from_protocol(raw: &str) -> Self {
        match raw {
            "stop" | "end_turn" => Self::EndTurn,
            "length" | "max_tokens" => Self::MaxTokens,
            "tool_calls" | "tool_use" => Self::ToolCalls,
            _ => Self::Other,
        }
    }

    /// 统一语义 → 协议原生字符串（[`Self::from_protocol`] 的逆映射，
    /// provider 层组装多轮消息时判断是否需要回喂工具结果）
    pub fn to_protocol(self) -> &'static str {
        match self {
            Self::EndTurn => "end_turn",
            Self::MaxTokens => "max_tokens",
            Self::ToolCalls => "tool_calls",
            Self::Other => "other",
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

    /// 模型要求调用工具（分片已在 provider 层聚合为完整调用；
    /// 紧跟 `Done { stop_reason: ToolCalls }`，调用方按 call_id 回填结果后继续对话）
    ToolCalls { calls: Vec<ToolCall> },

    /// 流正常结束
    Done { stop_reason: StopReason },
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod tests;
