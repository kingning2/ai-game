//! 房间路由：真实落库（SQLite）的房间创建 / 列表 / 详情。
//!
//! - `POST /api/rooms`        创建房间（名单写入 `room_players`，房间号 6 位）
//! - `GET  /api/rooms`        房间列表（新→旧）
//! - `GET  /api/rooms/{code}` 房间详情（含发牌后的真实身份——上帝视角）

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use state::AppState;

use crate::error::{ApiError, ApiResponse, ApiResult, AppError};

use super::werewolf::GameStatusDto;

/// POST /api/rooms 请求体
#[derive(Default, Deserialize, utoipa::ToSchema)]
pub struct RoomCreateRequest {
    /// 板子预设名（newbie_six / nine_standard / all_stars）
    pub board_id: String,
    /// 房间名（缺省「{board} 房间」）
    pub name: Option<String>,
    /// 参赛选手名（按座位序；长度须等于板子座位数）
    pub players: Vec<String>,
}

/// 房间席位
#[derive(Serialize, utoipa::ToSchema)]
pub struct RoomSeatDto {
    /// 座位号（1 起）
    pub seat: u8,
    /// 选手名
    pub name: String,
    /// 真实身份（发牌后才有；上帝视角）
    pub role: Option<String>,
    /// 是否存活
    pub alive: bool,
}

/// 房间信息
#[derive(Serialize, utoipa::ToSchema)]
pub struct RoomDto {
    /// 房间号（6 位大写，唯一）
    pub code: String,
    /// 房间名
    pub name: String,
    /// 板子预设名
    pub board_id: String,
    /// waiting / playing / finished
    pub status: String,
    pub created_at: String,
    /// 席位（详情接口才有）
    pub seats: Option<Vec<RoomSeatDto>>,
}

fn seat_dtos(seats: &[db::SeatRow]) -> Vec<RoomSeatDto> {
    seats
        .iter()
        .map(|s| RoomSeatDto {
            seat: s.seat,
            name: s.name.clone(),
            role: s.role.clone(),
            alive: s.alive,
        })
        .collect()
}

/// with_db 闭包错误：纯 SQL 错误 → 500；业务错误（如房间不存在）→ 透传
enum DbFlowError {
    Sql(rusqlite::Error),
    Api(AppError),
}

impl From<rusqlite::Error> for DbFlowError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sql(e)
    }
}

impl From<AppError> for DbFlowError {
    fn from(e: AppError) -> Self {
        Self::Api(e)
    }
}

impl From<db::DbError> for DbFlowError {
    fn from(e: db::DbError) -> Self {
        Self::Sql(e.into_inner())
    }
}

/// 借出数据库连接；未就绪（降级禁用）返回 503
fn with_db<T>(
    state: &AppState,
    f: impl FnOnce(&rusqlite::Connection) -> Result<T, DbFlowError>,
) -> ApiResult<T> {
    let guard = state
        .db
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(conn) = guard.as_ref() else {
        return Err(AppError::ServiceUnavailable.into());
    };
    f(conn)
        .map_err(|e| match e {
            DbFlowError::Sql(e) => AppError::Internal(Box::new(e)),
            DbFlowError::Api(e) => e,
        })
        .map_err(ApiError::from)
}

/// POST /api/rooms
/// 借出数据库连接执行 db 包函数（`DbError` → 500；连接不可用 → 503）。
/// 供 werewolf 路由做 `?room=` / `?game=` 定位与降级结算查询。
pub(crate) fn db_query<T>(
    state: &AppState,
    f: impl FnOnce(&rusqlite::Connection) -> Result<T, db::DbError>,
) -> ApiResult<T> {
    with_db(state, |conn| f(conn).map_err(DbFlowError::from))
}

#[utoipa::path(
    post,
    path = "/api/rooms",
    tag = "rooms",
    request_body = RoomCreateRequest,
    responses(
        (status = 200, description = "创建成功（房间号唯一）", body = ApiResponse<RoomDto>),
        (status = 400, description = "未知板子 / 名单长度不符", body = ApiResponse<String>),
        (status = 503, description = "数据库不可用", body = ApiResponse<String>)
    )
)]
pub async fn create_room(
    State(state): State<AppState>,
    body: Option<Json<RoomCreateRequest>>,
) -> ApiResult<Json<ApiResponse<RoomDto>>> {
    let request = body.map(|Json(r)| r).unwrap_or_default();
    let board = state::GameState::resolve_board(&request.board_id).ok_or_else(|| {
        AppError::BadRequest(format!(
            "未知板子预设：{}（可选 newbie_six / nine_standard / all_stars）",
            request.board_id
        ))
    })?;
    let total = usize::from(board.total());
    if request.players.len() != total {
        return Err(ApiError(AppError::BadRequest(format!(
            "名单长度不符：板子 {} 需要 {total} 人，收到 {} 人",
            request.board_id,
            request.players.len()
        ))));
    }
    let name = request
        .name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| {
            match request.board_id.as_str() {
                "newbie_six" => "新手 6 人局",
                "nine_standard" => "标准 9 人局",
                "all_stars" => "全明星 12 人局",
                other => other,
            }
            .to_string()
        });
    let seats: Vec<(u8, String)> = request
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| (u8::try_from(i + 1).unwrap_or(1), p.clone()))
        .collect();

    // 房间号唯一约束冲突时重试（6 位去掉易混淆字符，碰撞概率极低）
    let mut last_err: Option<ApiError> = None;
    for _ in 0..5 {
        let code = db::room_code();
        let result = with_db(&state, |conn| {
            db::create_room(conn, &code, &name, &request.board_id, &seats)?;
            Ok(code)
        });
        match result {
            Ok(code) => {
                return Ok(ApiResponse::ok(RoomDto {
                    code,
                    name,
                    board_id: request.board_id,
                    status: "waiting".into(),
                    created_at: String::new(),
                    seats: None,
                }));
            }
            Err(err @ ApiError(AppError::Internal(_))) => last_err = Some(err),
            Err(err) => return Err(err),
        }
    }
    Err(last_err.unwrap_or_else(|| ApiError(AppError::Internal("创建房间失败".into()))))
}

/// GET /api/rooms
#[utoipa::path(
    get,
    path = "/api/rooms",
    tag = "rooms",
    responses(
        (status = 200, description = "房间列表（新→旧）", body = ApiResponse<Vec<RoomDto>>),
        (status = 503, description = "数据库不可用", body = ApiResponse<String>)
    )
)]
pub async fn list_rooms(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<RoomDto>>>> {
    // 列表前先清扫孤儿房（等待超 10 分钟的永不开局垃圾）
    let rooms = with_db(&state, |conn| {
        db::sweep_waiting(conn, 10)?;
        Ok(db::list_rooms(conn, 50)?)
    })?;
    let dtos = rooms
        .into_iter()
        .map(|r| RoomDto {
            code: r.code,
            name: r.name,
            board_id: r.board_id,
            status: r.status,
            created_at: r.created_at,
            seats: None,
        })
        .collect();
    Ok(ApiResponse::ok(dtos))
}

/// GET /api/rooms/{code}
#[utoipa::path(
    get,
    path = "/api/rooms/{code}",
    tag = "rooms",
    params(("code" = String, Path, description = "房间号")),
    responses(
        (status = 200, description = "房间详情（含真实身份）", body = ApiResponse<RoomDto>),
        (status = 404, description = "房间不存在", body = ApiResponse<String>),
        (status = 503, description = "数据库不可用", body = ApiResponse<String>)
    )
)]
pub async fn get_room(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> ApiResult<Json<ApiResponse<RoomDto>>> {
    let found = with_db(&state, |conn| Ok(db::get_room(conn, &code)?))?
        .ok_or_else(|| AppError::NotFound(format!("房间 {code}")))?;
    Ok(ApiResponse::ok(RoomDto {
        code: found.0.code,
        name: found.0.name,
        board_id: found.0.board_id,
        status: found.0.status,
        created_at: found.0.created_at,
        seats: Some(seat_dtos(&found.1)),
    }))
}

/// 挂钩 werewolf start：房间身份落库 + 对局记录创建，返回 (room_id, game_id)
pub(crate) fn bind_start(
    state: &AppState,
    code: &str,
    seat_roles: &[(u8, String)],
) -> ApiResult<(i64, i64)> {
    with_db(state, |conn| {
        let (room, _) =
            db::get_room(conn, code)?.ok_or_else(|| AppError::NotFound(format!("房间 {code}")))?;
        db::save_deal(conn, room.id, seat_roles)?;
        let game_id = db::begin_game(conn, room.id, &room.board_id)?;
        Ok((room.id, game_id))
    })
}

/// 对局进行中增量落库事件流水；自然/手动结束后写胜负（幂等，轮询驱动）。
/// 裸局（db 无该 game 记录）不落库。
pub(crate) fn flush_progress(state: &AppState, game_id: i64, status: &GameStatusDto) {
    let _ = db_query(state, |conn| {
        let Some(game) = db::game_result(conn, game_id)? else {
            return Ok(()); // 裸局：无对局记录，不落库
        };
        let from = db::max_event_seq(conn, game_id)?;
        let fresh = status
            .events
            .iter()
            .skip(usize::try_from(from).unwrap_or(0))
            .map(|e| db::EventRow {
                seq: e.seq as i64,
                kind: e.kind.clone(),
                seat: e.seat,
                player: e
                    .seat
                    .and_then(|s| status.seats.get(usize::from(s)).cloned()),
                content: e.text.clone(),
                day: e.day,
            })
            .collect::<Vec<_>>();
        db::append_events(conn, game_id, &fresh)?;
        // 已写过胜负（含重启清扫标记 stale）的局不再重复落库
        if let (Some(winner), false) = (&status.winner, game.winner.is_some()) {
            let result = serde_json::json!({
                "ended_reason": status.ended_reason,
                "day": status.day,
                "roles": status.roles,
            });
            db::finish_game(conn, game_id, game.room_id, winner, &result.to_string())?;
            // 胜负首次落库：启动赛后 MVP 投票（独立任务，不阻塞轮询收尾；
            // 手动 end 无 winner、stale 清扫不走此分支，均不触发）
            crate::orchestr::mvp::spawn(state, game_id, game.room_id);
        }
        Ok(())
    });
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/rooms", get(list_rooms).post(create_room))
        .route("/api/rooms/{code}", get(get_room))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
