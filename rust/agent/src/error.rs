use http::StatusCode;
use thiserror::Error;

/// 通用 boxed 错误，用于网络层变体携带底层原因（与 server 侧 `AppError::Internal` 同一惯例）
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// agent 对接层统一错误。
///
/// 类型与构造策略都集中在这里（状态码映射 / body 截断 / 文案），
/// agent 包的各协议与 stream 层只负责检测并调用构造器，不自己拼错误。
/// 错误从 provider 冒出后，由调用方（server 路由层）映射成 HTTP 响应。
///
/// 网络层变体（[`AgentError::Request`] / [`AgentError::Stream`]）不依赖具体
/// HTTP 客户端类型，底层原因走 [`BoxError`] source 链携带——本包因此无需引入 reqwest。
#[derive(Debug, Error)]
pub enum AgentError {
    /// 运行配置不完整（如 API Key 未配置）。构建 provider 时不校验，
    /// 调用时才报——sidecar 未配 Key 也要能正常启动
    #[error("agent 配置不完整：{0}")]
    Config(String),

    /// 请求本身不合法（如 messages 为空 / 全是 system 消息）
    #[error("agent 请求不合法：{0}")]
    InvalidRequest(String),

    /// 请求发送失败（网络层：DNS / TCP / TLS / 连接超时），原因在 source 链
    #[error("agent 请求发送失败")]
    Request(#[source] BoxError),

    /// 上游返回非 2xx（鉴权失败 / 限流 / 模型不存在等），body 截断到 500 字符
    #[error("上游服务返回 {status}：{body}")]
    Upstream { status: StatusCode, body: String },

    /// HTTP 已 200 但上游在流中发来错误事件（此时无法再改 HTTP 状态码）
    #[error("上游流式返回错误：{0}")]
    UpstreamEvent(String),

    /// 流式响应读取中断（连接被断开 / 读 chunk 失败），原因在 source 链
    #[error("流式响应读取中断")]
    Stream(#[source] BoxError),

    /// SSE 帧无法解析为已知协议格式（上游行为异常）
    #[error("无法解析上游响应：{0}")]
    Protocol(String),
}

/// agent 对接链路的 Result 别名
pub type AgentResult<T> = Result<T, AgentError>;

impl AgentError {
    /// 非 2xx 响应 → [`AgentError::Upstream`]。
    /// body 截断到 500 字符的策略只在这里（防大 HTML 错误页刷屏）
    pub fn upstream(status: StatusCode, body: &str) -> Self {
        Self::Upstream {
            status,
            body: body.chars().take(500).collect(),
        }
    }

    /// API Key 未配置 → [`AgentError::Config`]（文案统一在这里，指向设置入口；
    /// Key 由前端设置页下发，不再从环境变量读取）
    pub fn missing_key() -> Self {
        Self::Config("API Key 未配置，请先在设置中完成配置".to_string())
    }

    /// messages 不满足协议要求 → [`AgentError::InvalidRequest`]（两家协议共用同一规则与文案）
    pub fn invalid_messages() -> Self {
        Self::InvalidRequest("messages 至少需要一条 user/assistant 消息".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Display 要带上关键上下文；source 链的覆盖在 agent 包的 e2e mock 测试里
    #[test]
    fn display_contains_context() {
        let err = AgentError::Upstream {
            status: StatusCode::UNAUTHORIZED,
            body: "invalid api key".into(),
        };
        assert!(err.to_string().contains("401"));
        assert!(err.to_string().contains("invalid api key"));

        let err = AgentError::missing_key();
        assert!(err.to_string().contains("API Key"));
    }

    /// 截断策略：按字符截（不切坏多字节中文），且只在这一个地方发生
    #[test]
    fn upstream_truncates_body_to_500_chars() {
        let body = "中".repeat(800);
        let err = AgentError::upstream(StatusCode::BAD_GATEWAY, &body);
        let AgentError::Upstream { body, .. } = err else {
            panic!("应为 Upstream 变体");
        };
        assert_eq!(body.chars().count(), 500);
    }

    #[test]
    fn missing_key_message_points_to_settings() {
        let err = AgentError::missing_key();
        assert!(matches!(err, AgentError::Config(message) if message.contains("未配置")));
    }
}
