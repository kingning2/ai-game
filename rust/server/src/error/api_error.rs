use super::app_error::AppError;
use super::response::ApiResponse;
use agent::AgentError;
use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::response::{IntoResponse, Response};
use state::StateError;

/// 传输层错误包装：AppError 负责领域语义，ApiError 负责渲染成 HTTP 响应（包裹 + 日志）。
///
/// 为 handler 的 `?` 便利，实现了常见来源的 From 转换。
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(error: AppError) -> Self {
        Self(error)
    }
}

/// agent 对接链路的错误：领域映射在 `app_error` 的 `From<AgentError> for AppError`，这里只负责包壳
impl From<AgentError> for ApiError {
    fn from(err: AgentError) -> Self {
        Self(err.into())
    }
}

/// state 层错误：NotFound 透传为 404，持久化失败归 Internal（上下文进 source 链只进日志）
impl From<StateError> for ApiError {
    fn from(err: StateError) -> Self {
        Self(err.into())
    }
}

impl From<QueryRejection> for ApiError {
    fn from(err: QueryRejection) -> Self {
        Self(err.into())
    }
}

impl From<JsonRejection> for ApiError {
    fn from(err: JsonRejection) -> Self {
        Self(err.into())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.0.status();
        let code = self.0.code();

        // 5xx 记录完整错误与 source 链；4xx 只记 warn 级摘要
        if status.is_server_error() {
            tracing::error!(code, error = ?self.0, "服务端错误");
        } else {
            tracing::warn!(code, error = %self.0, "客户端错误");
        }

        (
            status,
            axum::Json(ApiResponse::<()>::error(code, self.0.to_string())),
        )
            .into_response()
    }
}
