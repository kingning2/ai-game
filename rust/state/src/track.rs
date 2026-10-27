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
mod tests {
    use super::*;
    use crate::{AgentState, AppConfig, AppState};
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use axum::{Router, middleware, routing::get};
    use tower::ServiceExt;

    #[tokio::test]
    async fn tracks_all_requests() {
        let state = AppState::new(AppConfig::default(), AgentState::ephemeral());
        let app: Router<()> = Router::new()
            .route("/hi", get(|| async { "hi" }))
            // 无 fallback：未匹配路径走 axum 默认 404
            .layer(middleware::from_fn_with_state(state.clone(), track))
            .with_state(state.clone());

        let make = |uri: &str| HttpRequest::builder().uri(uri).body(Body::empty()).unwrap();

        app.clone().oneshot(make("/hi")).await.unwrap();
        app.oneshot(make("/nope")).await.unwrap();

        assert_eq!(state.stats.total(), 2, "404 fallback 请求也被统计");
        assert_eq!(state.stats.active(), 0);
        assert_eq!(state.stats.ok(), 1);
        assert_eq!(state.stats.client_errors(), 1);
    }
}
