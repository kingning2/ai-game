//! 胜利排行榜端点：跨对局聚合每位选手的参局数 / 胜场 / 胜率 / MVP 次数。
//!
//! - `GET /api/leaderboard`：榜单（胜场降序 → 胜率降序 → 参局数降序）；还没打完
//!   过任何一局时返回空数组
//!
//! 聚合逻辑与胜负判定（身份→阵营 vs games.winner）在 db 包 `leaderboard()`，
//! 这里只做 DTO 转换。前端 DTO 在 `apps/web` 手写对齐（字段变更时同步修改）。

use crate::error::{ApiResponse, ApiResult};
use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use state::AppState;

use super::rooms::db_query;

/// 排行榜行（GET 响应元素）
#[derive(Serialize, utoipa::ToSchema)]
pub struct LeaderboardEntryDto {
    /// 选手名
    pub name: String,
    /// 参与的已结束局数
    pub games: i64,
    /// 胜场数
    pub wins: i64,
    /// 当选 MVP 次数
    pub mvp_count: i64,
    /// 胜率（0..1）
    pub win_rate: f64,
}

/// GET /api/leaderboard
#[utoipa::path(
    get,
    path = "/api/leaderboard",
    tag = "werewolf",
    operation_id = "get_leaderboard",
    responses(
        (status = 200, description = "胜利排行榜（按胜场/胜率/参局数降序）", body = ApiResponse<Vec<LeaderboardEntryDto>>)
    )
)]
pub async fn get_leaderboard(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<LeaderboardEntryDto>>>> {
    // 仅统计正式选手（state::ROSTER_PLAYERS 白名单）：测试局临时玩家不入榜
    let rows = db_query(&state, |conn| db::leaderboard(conn, &state::ROSTER_PLAYERS))?;
    let entries = rows
        .into_iter()
        .map(|row| LeaderboardEntryDto {
            name: row.name,
            games: row.games,
            wins: row.wins,
            mvp_count: row.mvp_count,
            win_rate: row.win_rate,
        })
        .collect::<Vec<_>>();
    Ok(ApiResponse::ok(entries))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/leaderboard", get(get_leaderboard))
}
