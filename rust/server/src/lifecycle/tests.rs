//! mod.rs 的单元测试（由内联 mod tests 迁入）。

use super::build_router;
use state::{AgentState, AppConfig, AppState};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn routes_wired() {
    let app = build_router(AppState::new(AppConfig::default(), AgentState::ephemeral()));
    let get = |uri: &str| Request::builder().uri(uri).body(Body::empty()).unwrap();

    // health：裸 200
    let res = app.clone().oneshot(get("/api/health")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // info：包裹 200
    let res = app.clone().oneshot(get("/api/info")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // agent config：包裹 200（默认配置，Key 未配）
    let res = app.clone().oneshot(get("/api/agent/config")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // openapi 文档：JSON 200，含 agent 端点、不含文档端点自身
    let res = app
        .clone()
        .oneshot(get("/api-docs/openapi.json"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), 1 << 22)
        .await
        .unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(doc["paths"]["/api/agent/config"].is_object());
    assert!(doc["paths"]["/api-docs/openapi.json"].is_null());

    // 未知路径 → 404 包裹
    let res = app.clone().oneshot(get("/api/nope")).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // 方法不对 → 405 包裹
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
}
