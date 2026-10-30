//! mod.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use state::{AgentState, RosterState};
use tower::ServiceExt;

fn app(state: state::AppState) -> axum::Router {
    Router::new().merge(router()).with_state(state)
}

fn test_state() -> state::AppState {
    // 内存态 roster：不碰真实配置目录，测试间零串扰
    state::AppState::new_with_roster(
        state::AppConfig::default(),
        AgentState::ephemeral(),
        RosterState::ephemeral(),
    )
}

async fn get_json(state: state::AppState, uri: &str) -> (StatusCode, Value) {
    let res = app(state)
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}

async fn put_json(state: state::AppState, uri: &str, body: Value) -> (StatusCode, Value) {
    let res = app(state)
        .oneshot(
            Request::put(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

async fn delete_json(state: state::AppState, uri: &str) -> (StatusCode, Value) {
    let res = app(state)
        .oneshot(Request::delete(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn get_roster_lists_all_players_with_effective_config() {
    let (status, value) = get_json(test_state(), "/api/roster").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["code"], 0);
    let players = value["data"].as_array().unwrap();
    assert_eq!(players.len(), 12);
    assert_eq!(players[0]["name"], "Claude Opus 5.5");
    // 无覆盖时 has_override=false，effective 继承全局默认配置
    assert_eq!(players[0]["has_override"], false);
    assert!(players[0]["player_override"].is_null());
    assert!(
        players[0]["effective"]["openai"]["base_url"]
            .as_str()
            .unwrap()
            .starts_with("http")
    );
}

#[tokio::test]
async fn put_override_merges_and_masks_key() {
    let state = test_state();
    let (status, value) = put_json(
        state.clone(),
        "/api/roster/GPT-6.1",
        json!({"openai": {"model": "doubao-seed-1-6", "api_key": "sk-1234567890abcdef"}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value["code"], 0);
    let data = &value["data"];
    assert_eq!(data["has_override"], true);
    // 有效配置：覆盖的 model/api_key 生效，其余继承全局
    assert_eq!(data["effective"]["openai"]["model"], "doubao-seed-1-6");
    assert_eq!(data["effective"]["openai"]["api_key_masked"], "****cdef");
    assert_eq!(data["effective"]["openai"]["has_key"], true);
    // 覆盖明细回显脱敏：api_key 恒 null，脱敏值在 api_key_masked
    assert_eq!(
        data["player_override"]["openai"]["model"],
        "doubao-seed-1-6"
    );
    assert!(
        data["player_override"]["openai"]["api_key"].is_null(),
        "GET 不得回明文 Key"
    );
    assert_eq!(
        data["player_override"]["openai"]["api_key_masked"],
        "****cdef"
    );
}

#[tokio::test]
async fn put_thinking_roundtrip_and_validation() {
    let state = test_state();
    let (status, value) = put_json(
        state.clone(),
        "/api/roster/GPT-6.1",
        json!({"thinking": "enabled", "openai": {"model": "deepseek-v4-1-flash"}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value["data"]["player_override"]["thinking"], "enabled");

    // 非法档位 400
    let (status, _) = put_json(state, "/api/roster/GPT-6.1", json!({"thinking": "turbo"})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn put_empty_body_clears_override() {
    let state = test_state();
    let name = "/api/roster/Kimi%20K3.1%20Pro";
    let (_, value) = put_json(state.clone(), name, json!({"max_tokens": 4096})).await;
    assert_eq!(value["data"]["has_override"], true);

    let (_, value) = put_json(state, name, json!({})).await;
    assert_eq!(value["code"], 0);
    assert_eq!(value["data"]["has_override"], false);
    assert!(value["data"]["player_override"].is_null());
}

#[tokio::test]
async fn custom_player_can_be_created_listed_and_deleted() {
    let state = test_state();
    // PUT 名单外选手 = 自建入库（带头像）
    let (status, value) = put_json(
        state.clone(),
        "/api/roster/%E7%A5%9E%E7%A7%98%E5%98%89%E5%AE%BE",
        json!({"avatar": "🎭", "openai": {"model": "doubao-seed-2-1-pro"}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value["data"]["avatar"], "🎭");

    // GET 列表：内置 12 人 + 自建 1 人殿后
    let (_, value) = get_json(state.clone(), "/api/roster").await;
    let players = value["data"].as_array().unwrap();
    assert_eq!(players.len(), 13);
    assert_eq!(players[12]["name"], "神秘嘉宾");

    // 空名 400；内置选手删除 400
    let (status, _) = put_json(state.clone(), "/api/roster/%20%20", json!({})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = delete_json(state.clone(), "/api/roster/GPT-6.1").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 删除自建选手后列表回归 12 人
    let (_, value) = delete_json(state, "/api/roster/%E7%A5%9E%E7%A7%98%E5%98%89%E5%AE%BE").await;
    assert_eq!(value["code"], 0);
    assert_eq!(value["data"].as_array().unwrap().len(), 12);
}

#[tokio::test]
async fn invalid_base_url_returns_400() {
    let (status, _) = put_json(
        test_state(),
        "/api/roster/Claude%20Opus%205.5",
        json!({"openai": {"base_url": "ftp://bad"}}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
