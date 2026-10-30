//! tts.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use state::{AgentState, AppConfig};

fn test_state() -> AppState {
    AppState::new(AppConfig::default(), AgentState::ephemeral())
}

/// 带 DOUBAO_COOKIE 的测试态（仅用于不连网就能到达的分支）
fn test_state_with_cookie() -> AppState {
    AppState::new(
        AppConfig {
            doubao_cookie: Some("sessionid=test".to_string()),
            ..AppConfig::default()
        },
        AgentState::ephemeral(),
    )
}

/// status 接口：固定 13 个音色（主持人 + 12 选手），cache_dir 非空
/// （installed/phase 不断言——取决于本机是否已装模型）
#[tokio::test]
async fn status_lists_all_voices() {
    let app = Router::new().merge(router()).with_state(test_state());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tts/status")
                .body(Body::empty())
                .expect("请求"),
        )
        .await
        .expect("响应");
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("响应体");
    let body: Value = serde_json::from_slice(&bytes).expect("JSON");
    assert_eq!(body["code"], 0);
    let data = &body["data"];
    assert_eq!(data["voices"].as_array().map(Vec::len), Some(13));
    assert_eq!(data["voices"][0]["id"], "zm_100");
    assert_eq!(data["voices"][0]["label"], "主持人");
    assert_eq!(data["voices"][1]["label"], "豆包 Seed 2.1");
    assert!(data["cache_dir"].as_str().is_some_and(|s| !s.is_empty()));
    // 测试态 AppConfig::default() 未配 Cookie → 豆包引擎不可用
    assert_eq!(data["doubao_available"], false);
    assert!(matches!(
        data["phase"].as_str(),
        Some("idle" | "ready" | "failed" | "installing")
    ));
}

/// speak 未知音色 → 404（不碰模型资产，无需安装）
#[tokio::test]
async fn speak_unknown_voice_is_404() {
    let app = Router::new().merge(router()).with_state(test_state());
    let request = Request::builder()
        .method("POST")
        .uri("/api/tts/speak")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({ "voice": "不存在的音色", "text": "hi" }).to_string(),
        ))
        .expect("请求");
    let response = app.oneshot(request).await.expect("响应");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("响应体");
    let body: Value = serde_json::from_slice(&bytes).expect("JSON");
    assert_eq!(body["code"], 10004);
}

/// speak 空文本 → 400（参数校验先于模型检查）
#[tokio::test]
async fn speak_empty_text_is_400() {
    let app = Router::new().merge(router()).with_state(test_state());
    let request = Request::builder()
        .method("POST")
        .uri("/api/tts/speak")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({ "voice": "豆包 Seed 2.1", "text": "   " }).to_string(),
        ))
        .expect("请求");
    let response = app.oneshot(request).await.expect("响应");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("响应体");
    let body: Value = serde_json::from_slice(&bytes).expect("JSON");
    assert_eq!(body["code"], 10001);
}

/// 豆包引擎：未配 DOUBAO_COOKIE → 400 引导（不连网，配置检查先行）
#[tokio::test]
async fn speak_doubao_without_cookie_is_400() {
    let app = Router::new().merge(router()).with_state(test_state());
    let request = Request::builder()
        .method("POST")
        .uri("/api/tts/speak")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({ "voice": "taozi", "text": "你好", "engine": "doubao" }).to_string(),
        ))
        .expect("请求");
    let response = app.oneshot(request).await.expect("响应");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("响应体");
    let body: Value = serde_json::from_slice(&bytes).expect("JSON");
    assert_eq!(body["code"], 10001);
    assert!(
        body["message"]
            .as_str()
            .is_some_and(|m| m.contains("DOUBAO_COOKIE"))
    );
}

/// 豆包引擎：空文本 → 400（文本校验先行，不发起网络连接）
#[tokio::test]
async fn speak_doubao_empty_text_is_400() {
    let app = Router::new()
        .merge(router())
        .with_state(test_state_with_cookie());
    let request = Request::builder()
        .method("POST")
        .uri("/api/tts/speak")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({ "voice": "taozi", "text": "  ", "engine": "doubao" }).to_string(),
        ))
        .expect("请求");
    let response = app.oneshot(request).await.expect("响应");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("响应体");
    let body: Value = serde_json::from_slice(&bytes).expect("JSON");
    assert_eq!(body["code"], 10001);
}
