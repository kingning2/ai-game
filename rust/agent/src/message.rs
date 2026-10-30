use serde::{Deserialize, Serialize};

use crate::{AgentError, AgentResult};

/// 对话角色。Anthropic 协议不允许 system 出现在 messages 里——
/// 拆分逻辑在 [`crate::anthropic`]，本枚举保持两家并集语义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    /// 工具结果消息（OpenAI `role:"tool"` / Anthropic `tool_result` block；
    /// 序列化转换在各 provider 内完成）
    Tool,
}

/// 一条对话消息。多轮工具往返：
/// assistant 携带 [`Self::tool_calls`] 发起调用 → 下一条 Tool 消息用
/// [`ToolResult`] 按 `call_id` 回填结果 → 继续对话直到模型给出最终答复
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    /// assistant 发起的工具调用（其他角色恒空）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    /// Tool 角色携带的调用结果（其他角色恒空）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_results: Vec<ToolResult>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self::bare(Role::System, content)
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::bare(Role::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::bare(Role::Assistant, content)
    }

    /// 带工具调用的 assistant 消息（多轮工具往返的"我调了什么"）
    pub fn assistant_with_tools(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            tool_calls,
            ..Self::bare(Role::Assistant, content)
        }
    }

    /// 一条工具结果消息（`call_id` 对应 [`ToolCall::id`]）
    pub fn tool_result(call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_results: vec![ToolResult {
                call_id: call_id.into(),
                content: content.into(),
            }],
            ..Self::bare(Role::Tool, String::new())
        }
    }

    fn bare(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
        }
    }
}

/// 一个可被模型调用的工具（JSON Schema 描述参数）。
///
/// 序列化形态由各 provider 转换：OpenAI 包 `{"type":"function","function":{...}}`，
/// Anthropic 把 `parameters` 改名 `input_schema`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// 参数的 JSON Schema
    pub parameters: serde_json::Value,
}

impl ToolDefinition {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}

/// 模型发起的一次工具调用（流式分片已在 provider 层聚合完毕）。
///
/// `arguments` 保持未解析的 JSON 文本——参数校验归消费侧（各场景 schema 不同）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// 一次工具调用的结果（多轮工具往返：按 `call_id` 回喂给模型）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub content: String,
}

/// 工具调用策略（两家协议的取值映射在 provider 内完成：
/// Required → OpenAI `"required"` / Anthropic `{"type":"any"}`）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolChoice {
    #[default]
    Auto,
    /// 强制模型必须调用某个工具（结构化决策场景用）
    Required,
}

/// 思考模式（方舟统一参数 `thinking: {type}`；glm 系部分模型不支持 disabled，
/// 由 provider/网关决定是否报错，本层只透传意图）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingMode {
    Enabled,
    Disabled,
}

/// 一次流式对话请求
///
/// `model` 缺省用 provider 配置里的默认模型；`messages` 需含本次输入的完整历史，
/// 且至少一条 user/assistant 消息（provider 内部校验）；
/// `thinking` 缺省不发参数（跟随模型默认行为）；
/// `tools` 非空时模型可发起工具调用（产出 [`crate::AgentEvent::ToolCalls`] 事件）
#[derive(Debug, Clone, Default)]
pub struct ChatRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub thinking: Option<ThinkingMode>,
    pub tools: Vec<ToolDefinition>,
    pub tool_choice: ToolChoice,
}

impl ChatRequest {
    /// 公共校验：非空且至少一条 user/assistant（两家协议的硬性要求，
    /// 规则与文案统一在 [`AgentError::invalid_messages`]）
    pub fn validate(&self) -> AgentResult<()> {
        // 空列表时 any() 必为 false，无需单独判空
        let has_turn = self
            .messages
            .iter()
            .any(|message| matches!(message.role, Role::User | Role::Assistant));
        if !has_turn {
            return Err(AgentError::invalid_messages());
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "message_tests.rs"]
mod tests;
