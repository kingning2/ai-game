use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::app_state::AppState;

/// 请求统计中间件：进入时 `stats.begin()`，响应后按状态码 `stats.end()`
///
/// 用法（server main.rs，放**最外层**）：
/// `.layer(axum::middleware::from_fn(state::track))`
///
/// 最外层保证 panic 兜底（内层 CatchPanicLayer 转出的 500 响应）与 404 fallback 也被统计。
pub async fn track(State(state): State<AppState>, req: Request, next: Next) -> Response {
    state.stats.begin();
    let response = next.run(req).await;
    state.stats.end(response.status());
    response
}

#[cfg(test)]
#[path = "track_tests.rs"]
mod tests;
