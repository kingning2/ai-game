//! 接口文档：utoipa 聚合 OpenAPI 3.x，`/api-docs/openapi.json` 暴露给 Apifox 导入。
//!
//! Apifox 导入路径：项目设置 → 导入数据 → OpenAPI/Swagger →
//! URL 填 `http://<server>:<port>/api-docs/openapi.json`（也支持下载后上传文件）；
//! Apifox 的"定时导入"可基于同一 URL 自动同步。
//!
//! 维护规则：新增路由时 handler 必须带 `#[utoipa::path]`、DTO 必须 `ToSchema`，
//! 并把 path 加进下方 `paths(...)`——漏了文档就不含该端点。

use axum::{Json, Router, routing::get};
use state::AppState;
use utoipa::OpenApi;

use crate::routes::{agent_config, health, info, leaderboard, rooms, roster, tts, werewolf};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "agent-packages server",
        version = env!("CARGO_PKG_VERSION"),
        description = "agent-packages 桌面端伴生服务：agent 配置（前端设置页读写）、狼人杀对局控制面（开始/暂停/继续/结束/状态）与服务元信息，及本地语音合成（TTS）。统一响应包裹 ApiResponse<T>：code === 0 表示成功。"
    ),
    paths(
        health::health,
        info::info,
        agent_config::get_config,
        agent_config::put_config,
        roster::get_roster,
        roster::put_roster_player,
        rooms::create_room,
        rooms::list_rooms,
        rooms::get_room,
        leaderboard::get_leaderboard,
        werewolf::start_game,
        werewolf::pause_game,
        werewolf::resume_game,
        werewolf::end_game,
        werewolf::get_memory,
        werewolf::compact_memory,
        werewolf::get_status,
        werewolf::stream_status,
        tts::get_status,
        tts::setup,
        tts::speak,
    ),
    tags(
        (name = "agent", description = "agent 配置：设置页读写通道（Key 脱敏返回）"),
        (name = "roster", description = "选手级模型配置：12 名选手各自的 provider/模型/Key 覆盖，未覆盖字段继承全局 agent 配置"),
        (name = "rooms", description = "狼人杀房间（SQLite 落库）：创建 / 列表 / 详情；发牌后详情含真实身份"),
        (name = "werewolf", description = "狼人杀对局控制面：开始 / 暂停 / 继续 / 结束 / 状态查询；所有 POST 成功与 GET 均返回最新 GameStatusDto"),
        (name = "tts", description = "本地语音合成： kokoro 模型装配（status/setup）与台词合成（speak，返回 audio/wav）"),
        (name = "system", description = "探活与服务元信息"),
    ),
    // IntoParams 查询参数也必须在此注册，否则只生成 $ref 而无 schema 定义（文档与 codegen 都会悬空）
    components(schemas(werewolf::GameQuery, werewolf::MemoryQuery))
)]
pub struct ApiDoc;

/// GET /api-docs/openapi.json
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api-docs/openapi.json", get(openapi_json))
}

/// 离线导出契约：`cargo test -p server export_openapi` 把 openapi.json 写到 apps/web/，
/// 供前端 openapi-typescript 生成 schema.gen.ts（跑 `pnpm codegen` 一键完成两步）。
#[test]
fn export_openapi() {
    let json = serde_json::to_vec_pretty(&ApiDoc::openapi()).expect("serialize OpenAPI document");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../apps/web/openapi.json");
    std::fs::write(path, json).expect("write openapi.json");
}
