//! mod.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use state::{AgentState, AppConfig};
use tower::ServiceExt;

fn app(state: AppState) -> axum::Router {
    Router::new().merge(router()).with_state(state)
}

fn test_state() -> AppState {
    let state = AppState::new(AppConfig::default(), AgentState::ephemeral());
    // 内存库注入：不碰真实配置目录，测试间零串扰
    *state.db.lock().unwrap() = db::open_in_memory().ok();
    state
}

#[tokio::test]
async fn create_and_query_room() {
    let state = test_state();
    let router = app(state.clone());
    let body = json!({"board_id": "newbie_six", "name": "试炼房", "players": ["豆包","GPT-6.1","Kimi","Gemini","Claude","DeepSeek"]});
    let res = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rooms")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let value: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(value["code"], 0);
    let code = value["data"]["code"].as_str().unwrap().to_string();
    assert_eq!(code.len(), 6);

    // 列表 + 详情（详情含席位）
    let res = app(state.clone())
        .oneshot(
            Request::builder()
                .uri("/api/rooms")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = app(state)
        .oneshot(
            Request::builder()
                .uri(format!("/api/rooms/{code}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let seats = value["data"]["seats"].as_array().unwrap();
    assert_eq!(seats.len(), 6);
    assert_eq!(seats[0]["name"], "豆包");
    assert!(seats[0]["role"].is_null()); // 未发牌无身份
}

#[tokio::test]
async fn create_room_bad_players_is_400() {
    let state = test_state();
    let body = json!({"board_id": "newbie_six", "players": ["只有一个人"]});
    let res = app(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rooms")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_missing_room_is_404() {
    let state = test_state();
    let res = app(state)
        .oneshot(
            Request::builder()
                .uri("/api/rooms/ZZZZ99")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
