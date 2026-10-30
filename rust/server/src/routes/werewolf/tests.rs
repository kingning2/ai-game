//! mod.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::openapi as docs;
use state::{AgentState, AppConfig};

fn test_state() -> AppState {
    AppState::new(AppConfig::default(), AgentState::ephemeral())
}

/// 走完一条完整生命周期并断言各步响应
async fn send(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(json) => builder
            .header("content-type", "application/json")
            .body(Body::from(json.to_string())),
        None => builder.body(Body::empty()),
    }
    .expect("构造请求");
    let response = app.oneshot(request).await.expect("响应");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("读取响应体");
    (
        status,
        serde_json::from_slice(&bytes).expect("响应体应为 JSON"),
    )
}

#[tokio::test]
async fn game_lifecycle_end_to_end() {
    let state = test_state();
    let app = Router::new().merge(router()).with_state(state.clone());

    // 初始无对局：无参定位 → 404「没有进行中的对局」（多槽位下不再有全局 idle）
    let (status, body) = send(app, "GET", "/api/game/status", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 10004);

    // 开局（显式板子 + seed）
    let app = Router::new().merge(router()).with_state(state.clone());
    let (status, body) = send(
        app,
        "POST",
        "/api/game/start",
        Some(json!({"board": "nine_standard", "seed": 7})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["code"], 0);
    assert_eq!(body["data"]["status"], "running");
    assert_eq!(body["data"]["board"], "nine_standard");
    assert_eq!(body["data"]["seed"], 7);
    assert_eq!(body["data"]["total"], 9);
    assert_eq!(body["data"]["alive_total"], 9);
    assert_eq!(body["data"]["phase"], "night");
    assert_eq!(body["data"]["winner"], Value::Null);
    assert_eq!(
        body["data"]["roles"].as_array().expect("roles").len(),
        5,
        "9 人局：狼/预/女/猎/民 5 类"
    );

    // 重复开局：多槽位下每次 start 都是新裸局（latest 切到新局，不再 409）
    let app = Router::new().merge(router()).with_state(state.clone());
    let (status, body) = send(app, "POST", "/api/game/start", Some(json!({"seed": 99}))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["seed"], 99);

    // 暂停 → 继续（作用在最新局上）
    let app = Router::new().merge(router()).with_state(state.clone());
    let (status, body) = send(app, "POST", "/api/game/pause", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "paused");
    let app = Router::new().merge(router()).with_state(state.clone());
    let (status, body) = send(app, "POST", "/api/game/resume", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "running");

    // 直接结束 → ended + manual
    let app = Router::new().merge(router()).with_state(state.clone());
    let (status, body) = send(app, "POST", "/api/game/end", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "ended");
    assert_eq!(body["data"]["ended_reason"], "manual");

    // 结束后再结束 → 409；状态查询仍为 ended
    let app = Router::new().merge(router()).with_state(state.clone());
    let (status, _) = send(app, "POST", "/api/game/end", None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let app = Router::new().merge(router()).with_state(state.clone());
    let (status, body) = send(app, "GET", "/api/game/status", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "ended");
}

#[tokio::test]
async fn start_unknown_board_is_400() {
    let state = test_state();
    let app = Router::new().merge(router()).with_state(state);
    let (status, body) = send(
        app,
        "POST",
        "/api/game/start",
        Some(json!({"board": "nope"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 10001);
}

#[tokio::test]
async fn start_without_body_uses_defaults() {
    let state = test_state();
    let app = Router::new().merge(router()).with_state(state);
    let (status, body) = send(app, "POST", "/api/game/start", None).await;
    assert_eq!(status, StatusCode::OK, "空请求体应使用缺省板子");
    assert_eq!(body["data"]["board"], "nine_standard");
    assert!(
        body["data"]["seed"].as_u64().expect("seed") > 0,
        "缺省 seed 应随机生成"
    );
}

#[tokio::test]
async fn openapi_contains_game_paths() {
    let state = test_state();
    let app = Router::new().merge(docs::router()).with_state(state);
    let (_, body) = send(app, "GET", "/api-docs/openapi.json", None).await;
    let paths = body["paths"].as_object().expect("paths");
    for path in [
        "/api/game/start",
        "/api/game/pause",
        "/api/game/resume",
        "/api/game/end",
        "/api/game/status",
        "/api/game/memory",
        "/api/game/memory/compact",
    ] {
        assert!(paths.contains_key(path), "openapi 缺少 {path}");
    }
}

#[tokio::test]
async fn memory_requires_running_game() {
    let state = test_state();
    let app = Router::new().merge(router()).with_state(state);
    let (status, body) = send(app, "GET", "/api/game/memory?player=0", None).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "无对局应 404（多槽位定位失败）"
    );
    assert_eq!(body["code"], 10004);
}

#[tokio::test]
async fn memory_returns_player_view_after_start() {
    let state = test_state();
    let app = Router::new().merge(router()).with_state(state.clone());
    let (status, body) = send(
        app,
        "POST",
        "/api/game/start",
        Some(json!({"board": "nine_standard", "seed": 7})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // 第一夜刚开局：无发言/播报事实，记忆为空但结构完整
    let app = Router::new().merge(router()).with_state(state.clone());
    let (status, body) = send(app, "GET", "/api/game/memory?player=0", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["player"], "0");
    assert_eq!(body["data"]["entries"], json!([]));
    assert_eq!(body["data"]["rendered"], "");

    // 越界座位 → 400
    let app = Router::new().merge(router()).with_state(state);
    let (status, body) = send(app, "GET", "/api/game/memory?player=99", None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 10001);
}

#[tokio::test]
async fn compact_requires_running_game() {
    let state = test_state();
    let app = Router::new().merge(router()).with_state(state);
    let (status, body) = send(
        app,
        "POST",
        "/api/game/memory/compact",
        Some(json!({"player": 0})),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "无对局应 404（多槽位定位失败）"
    );
    assert_eq!(body["code"], 10004);
}

#[tokio::test]
async fn compact_below_threshold_skips_llm() {
    let state = test_state();
    let app = Router::new().merge(router()).with_state(state.clone());
    let (status, _) = send(
        app,
        "POST",
        "/api/game/start",
        Some(json!({"board": "nine_standard", "seed": 7})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 初始记忆 0 条，远低于缺省窗口 30：不调模型直接返回未压缩。
    // ephemeral AgentState 未配 Key——若真的调了模型会 400，断言 200 即证明未调
    let app = Router::new().merge(router()).with_state(state.clone());
    let (status, body) = send(
        app,
        "POST",
        "/api/game/memory/compact",
        Some(json!({"player": 0})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["compacted"], false);
    assert_eq!(body["data"]["entries"], 0);
    assert_eq!(body["data"]["summary"], serde_json::Value::Null);

    // 越界座位 → 400
    let app = Router::new().merge(router()).with_state(state);
    let (status, _) = send(
        app,
        "POST",
        "/api/game/memory/compact",
        Some(json!({"player": 99})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[test]
fn outcome_code_maps_winner_tag() {
    assert_eq!(outcome_code(&werewolf::Outcome::GoodWin), "good");
    let wolf = werewolf::Outcome::WerewolfWin {
        reason: werewolf::WerewolfReason::GodsWiped,
    };
    assert_eq!(outcome_code(&wolf), "werewolf");
}

/// 内存库注入版 AppState（多房间定位 / 降级结算测试用）
fn test_state_with_db() -> AppState {
    let state = AppState::new(AppConfig::default(), AgentState::ephemeral());
    *state.db.lock().unwrap() = db::open_in_memory().ok();
    state
}

fn seed_room(state: &AppState, code: &str) -> i64 {
    let guard = state.db.lock().unwrap();
    let conn = guard.as_ref().expect("内存库已注入");
    db::create_room(
        conn,
        code,
        "测试房",
        "newbie_six",
        &[
            (1, "甲".into()),
            (2, "乙".into()),
            (3, "丙".into()),
            (4, "丁".into()),
            (5, "戊".into()),
            (6, "己".into()),
        ],
    )
    .expect("建房成功")
}

#[tokio::test]
async fn rooms_get_their_own_games() {
    let state = test_state_with_db();
    seed_room(&state, "AAAAAA");
    seed_room(&state, "BBBBBB");
    let app = || Router::new().merge(router()).with_state(state.clone());

    // 双房并发开局：各自独立槽位，互不串局
    let (_, first) = send(
        app(),
        "POST",
        "/api/game/start",
        Some(json!({"board": "newbie_six", "seed": 7, "room_code": "AAAAAA"})),
    )
    .await;
    assert_eq!(first["data"]["seed"], 7, "{first}");
    let (_, second) = send(
        app(),
        "POST",
        "/api/game/start",
        Some(json!({"board": "newbie_six", "seed": 8, "room_code": "BBBBBB"})),
    )
    .await;
    assert_eq!(second["data"]["seed"], 8, "{second}");

    // ?room= 各自定位到本房对局
    let (_, s1) = send(app(), "GET", "/api/game/status?room=AAAAAA", None).await;
    assert_eq!(s1["data"]["seed"], 7, "{s1}");
    let (_, s2) = send(app(), "GET", "/api/game/status?room=BBBBBB", None).await;
    assert_eq!(s2["data"]["seed"], 8, "{s2}");
    // ?game= 直连对局 id（start 响应即该局 id）
    let game_id = second["data"]["seed"].as_u64().unwrap_or(0); // 占位，下面用 room 推导
    let _ = game_id;

    // 无参 → 最近启动的局（BBBBBB）
    let (_, latest) = send(app(), "GET", "/api/game/status", None).await;
    assert_eq!(latest["data"]["seed"], 8, "{latest}");

    // 暂停 A 不影响 B（?room= 定位各自生命周期）
    let (_, paused) = send(app(), "POST", "/api/game/pause?room=AAAAAA", None).await;
    assert_eq!(paused["data"]["status"], "paused", "{paused}");
    let (_, running) = send(app(), "GET", "/api/game/status?room=BBBBBB", None).await;
    assert_eq!(running["data"]["status"], "running", "{running}");
}

#[tokio::test]
async fn stale_room_returns_degraded_ended_status() {
    let state = test_state_with_db();
    let room_id = seed_room(&state, "CCCCCC");
    // 模拟重启前的对局：身份已落库、事件已落库、未写胜负（槽里没有这局）
    {
        let guard = state.db.lock().unwrap();
        let conn = guard.as_ref().expect("内存库已注入");
        db::save_deal(conn, room_id, &[(1, "狼人".into()), (2, "预言家".into())]).unwrap();
        let game_id = db::begin_game(conn, room_id, "newbie_six").unwrap();
        db::append_events(
            conn,
            game_id,
            &[db::EventRow {
                seq: 0,
                kind: "started".into(),
                seat: None,
                player: None,
                content: "开局：6 人".into(),
                day: 1,
            }],
        )
        .unwrap();
        // 开机清扫（bootstrap 同款）：未完局标记 stale
        assert_eq!(db::finish_stale_games(conn).unwrap(), 1);
    }

    let app = Router::new().merge(router()).with_state(state);
    let (status, body) = send(app, "GET", "/api/game/status?room=CCCCCC", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["status"], "ended");
    assert_eq!(body["data"]["stale"], true);
    assert_eq!(body["data"]["ended_reason"], "stale");
    assert_eq!(body["data"]["winner"], serde_json::Value::Null);
    let seats = body["data"]["seats"].as_array().expect("seats");
    assert_eq!(seats.len(), 6);
    assert_eq!(seats[0], "甲");
    let roles = body["data"]["seat_roles"].as_array().expect("roles");
    assert_eq!(roles[0], "狼人", "降级结算用真实身份");
    assert_eq!(roles[1], "预言家");
    assert_eq!(body["data"]["events"].as_array().expect("events").len(), 1);
}

#[tokio::test]
async fn locate_missing_room_or_game_is_404() {
    let state = test_state_with_db();
    let app = Router::new().merge(router()).with_state(state);
    let (status, body) = send(app, "GET", "/api/game/status?room=ZZZZ99", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["code"], 10004);

    let state = test_state_with_db();
    let app = Router::new().merge(router()).with_state(state);
    let (status, body) = send(app, "GET", "/api/game/status?game=424242", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["data"]["message"], serde_json::Value::Null);
}
