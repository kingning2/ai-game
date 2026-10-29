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
}

/// 一条对话消息（v1 纯文本；图片 / 工具消息待后续版本）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
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
/// `thinking` 缺省不发参数（跟随模型默认行为）
#[derive(Debug, Clone, Default)]
pub struct ChatRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub thinking: Option<ThinkingMode>,
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
mod tests {
    use super::*;

    /// serde 小写约定：server 的 DTO 直接复用本类型，字段值必须与各家协议一致
    #[test]
    fn role_serde_lowercase() {
        assert_eq!(serde_json::to_string(&Role::User).unwrap(), "\"user\"");
        let role: Role = serde_json::from_str("\"assistant\"").unwrap();
        assert_eq!(role, Role::Assistant);
    }

    #[test]
    fn constructors_set_roles() {
        assert_eq!(ChatMessage::system("s").role, Role::System);
        assert_eq!(ChatMessage::user("u").role, Role::User);
        assert_eq!(ChatMessage::assistant("a").role, Role::Assistant);
    }

    #[test]
    fn validate_rules() {
        let ok = ChatRequest {
            model: None,
            messages: vec![ChatMessage::user("hi")],
            thinking: None,
        };
        assert!(ok.validate().is_ok());

        let empty = ChatRequest::default();
        assert!(matches!(
            empty.validate(),
            Err(AgentError::InvalidRequest(_))
        ));

        let only_system = ChatRequest {
            model: None,
            thinking: None,
            messages: vec![ChatMessage::system("你是助手")],
        };
        assert!(matches!(
            only_system.validate(),
            Err(AgentError::InvalidRequest(_))
        ));
    }
}
