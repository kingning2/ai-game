use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::http::StatusCode;
use thiserror::Error;

use agent::AgentError;

/// 通用 boxed 错误，用于 Internal 携带底层原因
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// 应用错误枚举。错误码段约定：
/// - 10001~10099：客户端错误（4xx）
/// - 50001~：服务端错误（5xx）
///
/// code/status 的映射只在 [`AppError::code`] / [`AppError::status`] 一处维护，
/// 响应渲染（api_error）与单元测试都从这里取，禁止另写 match。
///
/// 部分变体（Unauthorized / Forbidden / PayloadTooLarge）暂无构造方，
/// 为接入认证 / 限流 / body 限制中间件预留。
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AppError {
    /// 400：参数取值/语义非法
    #[error("请求参数无效：{0}")]
    BadRequest(String),

    /// 401：未登录 / 凭证缺失或无效
    #[error("未授权访问")]
    Unauthorized,

    /// 403：已登录但无权限
    #[error("禁止访问")]
    Forbidden,

    /// 404：资源不存在
    #[error("资源不存在：{0}")]
    NotFound(String),

    /// 409：资源状态冲突（重复创建、并发版本冲突等）
    #[error("资源冲突：{0}")]
    Conflict(String),

    /// 405：HTTP 方法不支持
    #[error("该接口不支持此请求方法")]
    MethodNotAllowed,

    /// 413：请求体超出大小限制（配合 RequestBodyLimitLayer）
    #[error("请求体超出大小限制")]
    PayloadTooLarge,

    /// 415：Content-Type 不支持（如 JSON 接口收到的不是 application/json）
    #[error("不支持的媒体类型")]
    UnsupportedMediaType,

    /// 422：参数结构无法解析（JSON 语法/字段类型错误）
    #[error("请求体格式错误")]
    Unprocessable(String),

    /// 429：触发限流（配合限流层）
    #[error("请求过于频繁，请稍后重试")]
    TooManyRequests,

    /// 408：请求处理超时（配合超时层）
    #[error("请求处理超时")]
    Timeout,

    /// 500：服务内部错误。底层原因通过 source 链携带，只进日志、不回给客户端（防信息泄露）
    #[error("内部服务错误")]
    Internal(#[source] BoxError),

    /// 503：依赖服务不可用 / 维护中
    #[error("服务暂不可用，请稍后重试")]
    ServiceUnavailable,
}

impl AppError {
    /// 业务错误码（单一来源）
    pub fn code(&self) -> i32 {
        match self {
            Self::BadRequest(_) => 10001,
            Self::Unauthorized => 10002,
            Self::Forbidden => 10003,
            Self::NotFound(_) => 10004,
            Self::Conflict(_) => 10005,
            Self::MethodNotAllowed => 10006,
            Self::PayloadTooLarge => 10007,
            Self::UnsupportedMediaType => 10008,
            Self::Unprocessable(_) => 10009,
            Self::TooManyRequests => 10010,
            Self::Timeout => 10011,
            Self::Internal(_) => 50000,
            Self::ServiceUnavailable => 50001,
        }
    }

    /// HTTP 状态码（与 code 同源）
    pub fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::Unprocessable(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Self::Timeout => StatusCode::REQUEST_TIMEOUT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// 便捷构造 Internal（自动装箱任意错误类型）
    pub fn internal(err: impl Into<BoxError>) -> Self {
        Self::Internal(err.into())
    }
}

// ===== 框架层错误 → AppError 的转换链 =====
// 提取失败、io、超时等不需要每个 handler 手动 match，靠 From 自动接管。

/// JSON 提取失败：按 axum 给出的状态细分（415 内容类型错 / 422 结构错 / 400 其余）
impl From<JsonRejection> for AppError {
    fn from(err: JsonRejection) -> Self {
        match err.status() {
            StatusCode::UNSUPPORTED_MEDIA_TYPE => Self::UnsupportedMediaType,
            StatusCode::UNPROCESSABLE_ENTITY => Self::Unprocessable(err.body_text()),
            _ => Self::BadRequest(err.body_text()),
        }
    }
}

/// Query 提取失败（缺参数、类型不匹配）
impl From<QueryRejection> for AppError {
    fn from(err: QueryRejection) -> Self {
        Self::BadRequest(err.body_text())
    }
}

/// io 错误兜底归 Internal
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::internal(err)
    }
}

/// 超时（tokio::time::timeout 的 Elapsed）
impl From<tokio::time::error::Elapsed> for AppError {
    fn from(_err: tokio::time::error::Elapsed) -> Self {
        Self::Timeout
    }
}

// ===== 领域错误 → AppError 的语义映射 =====

/// agent 对接链路的领域映射（server 侧 `?` 便利的底层）。
///
/// 映射原则：
/// - 客户端输入问题 → 4xx；服务端/上游问题 → 5xx
/// - 上游 4xx 意味着我们的凭证或请求形状有问题，归 Internal 而不是透传给客户端
/// - 网络层失败（发不出去 / 中途断开）统一归 ServiceUnavailable
impl From<AgentError> for AppError {
    fn from(err: AgentError) -> Self {
        match err {
            AgentError::InvalidRequest(message) => Self::BadRequest(message),
            AgentError::Upstream {
                status: StatusCode::TOO_MANY_REQUESTS,
                ..
            } => Self::TooManyRequests,
            AgentError::Upstream { status, .. } if status.is_server_error() => {
                Self::ServiceUnavailable
            }
            // 网络层失败（发不出去 / 中途断开 / 流中错误）→ 上游不可用
            AgentError::Request(_) | AgentError::Stream(_) | AgentError::UpstreamEvent(_) => {
                Self::ServiceUnavailable
            }
            // Config / Upstream(4xx) / Protocol：完整上下文进 source 链，只进日志
            agent_error => Self::internal(agent_error),
        }
    }
}

/// state 持久化/名单错误 → AppError
impl From<state::StateError> for AppError {
    fn from(err: state::StateError) -> Self {
        match err {
            state::StateError::Invalid(message) => Self::BadRequest(message),
            // 持久化失败：完整上下文与底层原因进 source 链，只进日志
            persist_error => Self::internal(persist_error),
        }
    }
}

#[cfg(test)]
#[path = "app_error_tests.rs"]
mod tests;
