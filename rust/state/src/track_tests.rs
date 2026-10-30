//! track.rs 的单元测试（由内联 mod tests 迁入）。

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
