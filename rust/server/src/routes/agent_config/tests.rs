//! mod.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use state::AgentState;
use tower::ServiceExt;

fn app(state: state::AppState) -> axum::Router {
    Router::new().merge(router()).with_state(state)
}

fn test_state() -> state::AppState {
    AppState::new(state::AppConfig::default(), AgentState::ephemeral())
}

#[tokio::test]
async fn get_config_masks_key() {
    let state = test_state();
    // 预置一个 Key，验证 GET 只出脱敏形态
    let mut config = agent::AgentConfig::default();
    config.openai.api_key = "sk-1234567890abcdef".to_string();
    state.agent.update(config).unwrap();

    let response = app(state)
        .oneshot(
            Request::builder()
                .uri("/api/agent/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1 << 20)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["code"], 0);
    let masked = value["data"]["openai"]["api_key_masked"].as_str().unwrap();
    assert_eq!(masked, "****cdef");
    assert_eq!(value["data"]["openai"]["has_key"], true);
    // 明文绝不出现在响应里
    assert!(!String::from_utf8_lossy(&body).contains("abcdef"));
}

#[tokio::test]
async fn put_config_updates_and_keeps_key_on_absent() {
    // 先配 Key
    let mut config = agent::AgentConfig::default();
    config.openai.api_key = "sk-keep-me-secret".to_string();
    let agent_state = AgentState::ephemeral();
    agent_state.update(config).unwrap();
    let state = AppState::new(state::AppConfig::default(), agent_state);

    // 只改 model，不带 api_key → Key 保留
    let response = app(state.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/agent/config")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"openai": {"model": "gpt-4o"}}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let current = state.agent.config();
    assert_eq!(current.openai.model, "gpt-4o");
    assert_eq!(current.openai.api_key, "sk-keep-me-secret");
}

#[tokio::test]
async fn put_config_rejects_bad_base_url() {
    let response = app(test_state())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/agent/config")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"openai": {"base_url": "not-a-url"}}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn mask_key_rules() {
    assert_eq!(mask_key(""), "");
    assert_eq!(mask_key("short"), "****");
    assert_eq!(mask_key("sk-1234567890abcdef"), "****cdef");
}

#[test]
fn validate_accepts_default_config() {
    assert!(validate(&AgentConfig::default()).is_ok());
}
