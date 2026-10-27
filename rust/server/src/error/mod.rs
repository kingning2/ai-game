//! 传输层错误与响应包裹（原 workspace error 包的 server 侧部分收敛于此）。
//!
//! - [`app_error`]：[`AppError`](app_error::AppError) 错误枚举 + 各来源的领域映射
//! - [`api_error`]：[`ApiError`](api_error::ApiError) 渲染成 HTTP 响应（包裹 + 日志）
//! - [`response`]：[`ApiResponse`](response::ApiResponse) 统一成功/错误包裹

mod api_error;
mod app_error;
mod response;

pub use api_error::ApiError;
pub use app_error::{AppError, BoxError};
pub use response::ApiResponse;

/// handler 返回值别名：`ApiResult<T>` = `Ok(T)` / `Err(ApiError)`
pub type ApiResult<T> = Result<T, ApiError>;

/// `Router::fallback` 用：未匹配路径统一返回 404 包裹（否则 axum 默认回裸文本，前端会归为 http 错误而非 business）
pub async fn fallback_404() -> ApiError {
    ApiError(AppError::NotFound("接口不存在".into()))
}

/// `Router::method_not_allowed_fallback` 用：路径存在但方法不对，统一返回 405 包裹
pub async fn fallback_405() -> ApiError {
    ApiError(AppError::MethodNotAllowed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// fallback 的语义正确性
    #[tokio::test]
    async fn fallback_semantics() {
        assert_eq!(
            fallback_404().await.0.status(),
            axum::http::StatusCode::NOT_FOUND
        );
        assert_eq!(
            fallback_405().await.0.status(),
            axum::http::StatusCode::METHOD_NOT_ALLOWED
        );
    }
}
