//! 对局控制面端点：开始 / 暂停 / 继续 / 结束 / 状态查询 / 玩家记忆查询。
//!
//! - `POST /api/game/start`：开局（板子预设 + 可选 seed，可复现）
//! - `POST /api/game/pause`：暂停（引擎保留）
//! - `POST /api/game/resume`：暂停后继续
//! - `POST /api/game/end`：直接结束（强制终局）
//! - `GET  /api/game/status`：当前对局状态与局面总览
//! - `GET  /api/game/stream`：SSE 实时事件流（变化才推全量快照；前端对局页订阅源）
//! - `GET  /api/game/memory?player=N`：N 号视角的对局记忆（编排层抽取，供 LLM 决策 prompt）
//! - `POST /api/game/memory/compact`：记忆过长时调 LLM 压缩老段为摘要
//!   （划分 → LLM → 落回，结果跨请求存活；未超窗不调模型）
//!
//! 生命周期非法转移返回 409（如 running 时再 start、idle 时 pause）。
//! Apifox：`GET /api-docs/openapi.json` 的 `game` tag 已含全部端点。

use crate::error::{ApiError, ApiResponse, ApiResult, AppError};
use axum::{
    Json, Router,
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use state::{AppState, GameError, GameState, GameStatus, GameView, InvalidTransition};
use werewolf::{Action, DeathCause, GameEngine, GameEvent, Outcome, PlayerId, Role};

/// game 路由定位参数：`?room=房间码`（优先）或 `?game=对局 id`；都缺省用最近启动的对局
#[derive(Default, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct GameQuery {
    /// 房间码（优先于 game；按 db 该房间最近一局定位）
    pub room: Option<String>,
    /// 对局 id（裸局为负数自增 id）
    pub game: Option<i64>,
}

/// POST /api/game/start 请求体：全字段可选（缺省 = 全明星板 + 随机种子）
#[derive(Default, Deserialize, utoipa::ToSchema)]
pub struct GameStartRequest {
    /// 板子预设：`newbie_six`（新手 6 人局）/ `nine_standard`（经典 9 人局，缺省）/ `all_stars`（全明星 12 人局）
    pub board: Option<String>,
    /// 随机种子：缺省或 0 = 服务端随机生成；同种子同发牌（可复现）
    pub seed: Option<u64>,
    /// 座位 → 选手名（长度须等于座位数；缺省取选手名册前 N 位）。
    /// LLM 决策按名取选手模型配置
    pub players: Option<Vec<String>>,
    /// 房间号（SQLite 房间；传入则身份落库 + 事件流水关联该房间）
    pub room_code: Option<String>,
}

/// 单个身份的存活统计
#[derive(Serialize, utoipa::ToSchema)]
pub struct RoleStatusDto {
    /// 角色代码：werewolf / seer / witch / guard / hunter / idiot / knight / villager
    pub role: String,
    /// 展示名：狼人 / 预言家…
    pub name: String,
    /// 该身份总人数
    pub total: u8,
    /// 仍存活人数
    pub alive: u8,
}

/// GET /api/game/status 响应（各 POST 端点同样返回本结构：操作后的最新状态）
#[derive(Serialize, utoipa::ToSchema)]
pub struct GameStatusDto {
    /// 生命周期：idle / running / paused / ended
    pub status: String,
    /// 板子预设名；未开局为 null
    pub board: Option<String>,
    /// 开局种子
    pub seed: u64,
    /// 当前天数（第一夜为 1）；未开局为 0
    pub day: u32,
    /// 局面阶段：idle / night / day_speech / knight_duel / day_vote / hunter_shot / settled
    pub phase: String,
    /// 座位总数
    pub total: u8,
    /// 存活人数
    pub alive_total: u8,
    /// 各身份存活统计
    pub roles: Vec<RoleStatusDto>,
    /// 胜方：good / werewolf；仅引擎自然结算后有值
    pub winner: Option<String>,
    /// 结束原因：settled（自然分出胜负）/ manual（手动结束）
    pub ended_reason: Option<String>,
    /// 座位 → 选手名（未开局为空）
    pub seats: Vec<String>,
    /// 座位 → 角色显示名（上帝视角；未开局为空）
    pub seat_roles: Vec<String>,
    /// 座位 → Edge TTS 音色（观战语音播报；主持人音色前端固定；未分配为空）
    pub seat_voices: Vec<String>,
    /// 服务器重启导致的中断标记（重启后历史局的降级结算为 true；实时视图为 null）
    pub stale: Option<bool>,
    /// 赛后 MVP 投票（无 MVP 流程的局为 null；见 [`MvpDto`]）
    pub mvp: Option<MvpDto>,
    /// 上帝视角事件流（按发生顺序）；未开局为空
    pub events: Vec<GameEventDto>,
}

/// 单条事件（上帝视角；隐私事件同样包含，观战界面自行取舍展示）
#[derive(Serialize, utoipa::ToSchema)]
pub struct GameEventDto {
    /// 事件序号（从 0 递增，即引擎事件数组下标）
    pub seq: usize,
    /// 天数
    pub day: u32,
    /// 事件类型：started / night / night_action / wolf_chat / wolf_pass / dawn /
    /// speech / duel / vote / idiot / hunter / died / over
    pub kind: String,
    /// 主角座位（无单一主角的事件为 null）
    pub seat: Option<u8>,
    /// 面向观战者的中文描述
    pub text: String,
    /// dawn：昨夜死亡座位列表（平安夜为空）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deaths: Vec<u8>,
    /// vote：平票（无人放逐）
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tie: bool,
    /// vote 放逐座位 / hunter 枪击目标座位 / duel 败者座位
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<u8>,
}

/// 上帝视角事件渲染：`(day, kind, seat, 文本)`；未知事件返回 `None`
type RenderedEvent = (u32, &'static str, Option<PlayerId>, String);

pub fn render_event(event: &GameEvent, secret_visible: bool) -> Option<RenderedEvent> {
    /// 引擎座位号 0 起，面向人（主持词/日志/前端匹配）一律 1 起
    fn n(id: PlayerId) -> PlayerId {
        id + 1
    }
    match event {
        GameEvent::GameStarted { players, board } => {
            let roles = board
                .iter()
                .map(|(name, count)| format!("{name}x{count}"))
                .collect::<Vec<_>>()
                .join(" ");
            Some((
                u32::from(*players),
                "started",
                None,
                format!("开局：{players} 人（{roles}）"),
            ))
        }
        GameEvent::NightBegan { day } => Some((*day, "night", None, format!("第 {day} 夜天黑"))),
        GameEvent::NightAction { day, actor, action } => secret_visible.then(|| {
            (
                *day,
                "night_action",
                Some(n(actor.id)),
                format!(
                    "{}号（{}）：{}",
                    n(actor.id),
                    actor.role.display_name(),
                    action_text(action)
                ),
            )
        }),
        GameEvent::WolfChatSpoke { day, speaker, text } => secret_visible.then(|| {
            (
                *day,
                "wolf_chat",
                Some(n(speaker.id)),
                format!("{}号（狼）耳语：{text}", n(speaker.id)),
            )
        }),
        GameEvent::WolfChatPassed { day, speaker } => secret_visible.then(|| {
            (
                *day,
                "wolf_pass",
                Some(n(speaker.id)),
                format!("{}号（狼）保持沉默", n(speaker.id)),
            )
        }),
        GameEvent::Dawn { day, deaths } => {
            let text = if deaths.is_empty() {
                "平安夜".to_string()
            } else {
                let names = deaths
                    .iter()
                    .map(|d| format!("{}号（{}）", n(d.player), death_text(d.cause)))
                    .collect::<Vec<_>>()
                    .join("、");
                format!("昨夜死亡：{names}")
            };
            Some((*day, "dawn", None, text))
        }
        GameEvent::SpeechMade { day, speaker, text } => {
            Some((*day, "speech", Some(n(*speaker)), text.clone()))
        }
        GameEvent::KnightDuelResolved {
            day,
            knight,
            target,
            target_is_wolf,
            loser,
        } => Some((
            *day,
            "duel",
            Some(n(*knight)),
            format!(
                "{}号决斗{}号：{}（{}是狼人）出局",
                n(*knight),
                n(*target),
                loser.role.display_name(),
                if *target_is_wolf {
                    "对方"
                } else {
                    "骑士猜错，自己"
                }
            ),
        )),
        GameEvent::VoteResolved {
            day,
            counts,
            tie,
            exiled,
        } => {
            let votes = counts
                .iter()
                .map(|(seat, v)| format!("{}号x{v}", n(*seat)))
                .collect::<Vec<_>>()
                .join("、");
            let text = if *tie {
                format!("平票，无人放逐（{votes}）")
            } else {
                format!(
                    "放逐{}号（{}）；{votes}",
                    n(exiled.as_ref().map(|p| p.id).unwrap_or_default()),
                    exiled
                        .as_ref()
                        .map(|p| p.role.display_name())
                        .unwrap_or("？")
                )
            };
            Some((*day, "vote", None, text))
        }
        GameEvent::IdiotRevealed { day, idiot } => Some((
            *day,
            "idiot",
            Some(n(*idiot)),
            format!("{}号白痴翻牌免死，失去投票权", n(*idiot)),
        )),
        GameEvent::HunterShotFired {
            day,
            hunter,
            target,
        } => Some((
            *day,
            "hunter",
            Some(n(hunter.id)),
            match target {
                Some(t) => format!(
                    "{}号（{}）开枪带走{}号（{}）",
                    n(hunter.id),
                    hunter.role.display_name(),
                    n(t.id),
                    t.role.display_name()
                ),
                None => format!(
                    "{}号（{}）放弃开枪",
                    n(hunter.id),
                    hunter.role.display_name()
                ),
            },
        )),
        GameEvent::PlayerDied { day, player, cause } => Some((
            *day,
            "died",
            Some(n(player.id)),
            format!(
                "{}号（{}）出局：{}",
                n(player.id),
                player.role.display_name(),
                death_text(*cause)
            ),
        )),
        GameEvent::GameOver { outcome } => Some((
            u32::MAX,
            "over",
            None,
            match outcome {
                Outcome::WerewolfWin { .. } => "狼人阵营获胜".to_string(),
                Outcome::GoodWin => "好人阵营获胜".to_string(),
            },
        )),
    }
}

/// 对局流水落终端（上帝视角，含狼聊/夜刀）：操作员在 server 控制台看完整时间线
pub fn log_new_events(engine: &GameEngine, from: usize) {
    for event in engine.events().iter().skip(from) {
        let Some((day, kind, _, text)) = render_event(event, true) else {
            continue;
        };
        let day_label = if day == u32::MAX {
            "终局".to_string()
        } else {
            format!("第{day}天")
        };
        tracing::info!("[对局] {day_label} [{kind}] {text}");
    }
}

/// 夜晚动作的简短描述
fn action_text(action: &Action) -> String {
    match action {
        Action::GuardProtect { target } => match target {
            Some(t) => format!("守护{t}号"),
            None => "今晚不守".to_string(),
        },
        Action::WolfChat { text } => match text {
            Some(t) => format!("耳语「{t}」"),
            None => "保持沉默".to_string(),
        },
        Action::WerewolfKill { target } => format!("狼刀{target}号"),
        Action::WitchAct { save, poison } => {
            let save = if *save { "用解药" } else { "不用解药" };
            let poison = match poison {
                Some(t) => format!("，毒{t}号"),
                None => "，不毒人".to_string(),
            };
            format!("{save}{poison}")
        }
        Action::SeerCheck { target } => format!("查验{target}号"),
        Action::Speech { text } => format!("发言「{text}」"),
        Action::Vote { target } => match target {
            Some(t) => format!("投{t}号"),
            None => "弃票".to_string(),
        },
        Action::HunterShot { target } => match target {
            Some(t) => format!("枪{t}号"),
            None => "放弃开枪".to_string(),
        },
        Action::KnightDuel { target } => match target {
            Some(t) => format!("决斗{t}号"),
            None => "放弃决斗".to_string(),
        },
    }
}

/// 死因描述
fn death_text(cause: DeathCause) -> &'static str {
    match cause {
        DeathCause::Slain => "夜间遇害",
        DeathCause::Poisoned => "中毒身亡",
        DeathCause::Exiled => "投票放逐",
        DeathCause::Shot => "中枪出局",
        DeathCause::Duel => "决斗落败",
    }
}

/// 角色代码（接口字符串形态）
fn role_code(role: Role) -> &'static str {
    match role {
        Role::Werewolf => "werewolf",
        Role::Seer => "seer",
        Role::Witch => "witch",
        Role::Guard => "guard",
        Role::Hunter => "hunter",
        Role::Idiot => "idiot",
        Role::Knight => "knight",
        Role::Villager => "villager",
    }
}

/// 赛后 MVP 投票状态（仅该局有 MVP 流程时非空；手动终局 / stale 降级局为 null）
#[derive(Serialize, utoipa::ToSchema)]
pub struct MvpDto {
    /// 投票阶段：pending（投票中）→ done（已决出，选票从 db 读取）
    pub phase: String,
    /// 本场 MVP 选手名；pending 时为 null
    pub winner: Option<String>,
    /// 全部有效选票（done 后非空）
    pub votes: Vec<MvpVoteDto>,
}

/// 单张 MVP 选票
#[derive(Serialize, utoipa::ToSchema)]
pub struct MvpVoteDto {
    /// 投票人
    pub voter: String,
    /// 被投人
    pub candidate: String,
    /// 投票理由（LLM 生成，可能为空）
    pub reason: String,
}

impl GameStatusDto {
    /// 从控制面视图派生接口形态
    pub fn of(view: &GameView) -> Self {
        let (total, alive_total, roles, day) = match &view.summary {
            Some(summary) => (
                summary.total,
                summary.alive_total,
                summary
                    .roles
                    .iter()
                    .map(|r| RoleStatusDto {
                        role: role_code(r.role).to_string(),
                        name: r.role.display_name().to_string(),
                        total: r.total,
                        alive: r.alive,
                    })
                    .collect(),
                summary.day,
            ),
            None => (0, 0, Vec::new(), 0),
        };
        Self {
            status: view.status.as_str().to_string(),
            board: view.board.clone(),
            seed: view.seed,
            seat_voices: view.seat_voices.clone(),
            day,
            phase: view.phase.to_string(),
            total,
            alive_total,
            roles,
            winner: view.outcome.as_ref().map(outcome_code),
            ended_reason: if view.status == GameStatus::Ended {
                Some(
                    if view.outcome.is_some() {
                        "settled"
                    } else {
                        "manual"
                    }
                    .to_string(),
                )
            } else {
                None
            },
            seats: view.seats.clone(),
            seat_roles: view.seat_roles.clone(),
            stale: None,
            mvp: None,
            events: Vec::new(),
        }
    }

    /// 附加上帝视角事件流（开局后才有引擎可取；只读事件切片，不克隆引擎）
    pub fn with_events(mut self, events: Option<&[GameEvent]>) -> Self {
        if let Some(events) = events {
            self.events = events
                .iter()
                .enumerate()
                .filter_map(|(seq, event)| {
                    render_event(event, true).map(|(day, kind, seat, text)| {
                        // 结构化事实：文本面向人眼，座位号/平票等数字面向前端逻辑
                        let (deaths, tie, target) = match event {
                            GameEvent::Dawn { deaths, .. } => {
                                (deaths.iter().map(|d| d.player + 1).collect(), false, None)
                            }
                            GameEvent::VoteResolved { tie, exiled, .. } => {
                                (Vec::new(), *tie, exiled.as_ref().map(|p| p.id + 1))
                            }
                            GameEvent::HunterShotFired { target, .. } => {
                                (Vec::new(), false, target.as_ref().map(|t| t.id + 1))
                            }
                            GameEvent::KnightDuelResolved { loser, .. } => {
                                (Vec::new(), false, Some(loser.id + 1))
                            }
                            _ => (Vec::new(), false, None),
                        };
                        GameEventDto {
                            seq,
                            day,
                            kind: kind.to_string(),
                            seat,
                            text,
                            deaths,
                            tie,
                            target,
                        }
                    })
                })
                .collect();
        }
        self
    }
}

/// 生命周期非法转移 → 409
fn conflict(err: InvalidTransition) -> AppError {
    AppError::Conflict(err.to_string())
}

/// 槽位错误映射：`Config`（对局不存在或已失效）→ 404，非法转移 → 409
fn slot_err(err: GameError) -> AppError {
    match err {
        GameError::Config(message) => AppError::NotFound(message),
        GameError::Transition(transition) => conflict(transition),
    }
}

/// 定位目标对局：`?room=`（db 该房间最近一局）优先，其次 `?game=`，都缺省用最近启动局
fn locate_game(state: &AppState, query: &GameQuery) -> ApiResult<i64> {
    if let Some(code) = &query.room {
        let found =
            crate::routes::rooms::db_query(state, |conn| db::latest_game_by_room(conn, code))?
                .ok_or_else(|| AppError::NotFound(format!("房间 {code} 不存在")))?;
        return Ok(found.game_id);
    }
    if let Some(game_id) = query.game {
        return Ok(game_id);
    }
    state
        .game
        .latest()
        .ok_or_else(|| ApiError(AppError::NotFound("没有进行中的对局".into())))
}

/// seed 缺省或 0 时用时钟纳秒生成（零依赖随机；同种子恒同发牌）
fn resolve_seed(seed: Option<u64>) -> u64 {
    seed.filter(|s| *s != 0).unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0x9E37_79B9_7F4A_7C15, |d| d.as_nanos() as u64)
    })
}

/// Outcome 的接口字符串（胜方阵营：good / werewolf）
fn outcome_code(outcome: &Outcome) -> String {
    match outcome {
        Outcome::GoodWin => "good".to_string(),
        Outcome::WerewolfWin { .. } => "werewolf".to_string(),
    }
}

/// POST /api/game/start
#[utoipa::path(
    post,
    path = "/api/game/start",
    tag = "game",
    operation_id = "start_game",
    request_body = Option<GameStartRequest>,
    responses(
        (status = 200, description = "开局成功，返回开局后状态", body = ApiResponse<GameStatusDto>),
        (status = 400, description = "未知板子预设", body = ApiResponse<String>),
        (status = 409, description = "已有对局进行中 / 暂停中", body = ApiResponse<String>)
    )
)]
pub async fn start_game(
    State(state): State<AppState>,
    body: Option<Json<GameStartRequest>>,
) -> ApiResult<Json<ApiResponse<GameStatusDto>>> {
    let request = body.map(|Json(r)| r).unwrap_or_default();
    let board_name = request.board.unwrap_or_else(|| "nine_standard".to_string());
    let board = GameState::resolve_board(&board_name).ok_or_else(|| {
        AppError::BadRequest(format!(
            "未知板子预设：{board_name}（可选 newbie_six / nine_standard / all_stars）"
        ))
    })?;
    let seed = resolve_seed(request.seed);
    let engine = GameEngine::new(board, seed)
        .map_err(|err| AppError::BadRequest(format!("板子配置非法：{err}")))?;
    // 开局流水落终端（含 GameStarted；后续事件由 autoplay 每步喂入后追加输出）
    log_new_events(&engine, 0);
    // 座位 → 选手名：请求指定优先，缺省取选手名册前 N 位
    let total = engine.total_players();
    let seats = request
        .players
        .filter(|players| players.len() == total as usize)
        .unwrap_or_else(|| {
            werewolf_roster_names()
                .into_iter()
                .take(total as usize)
                .collect()
        });
    // 座位 → 真实身份显示名（引擎已发牌；开局前落库，好用 db 的 game_id 作槽位键）
    let seat_roles: Vec<(u8, String)> = engine
        .players()
        .iter()
        .enumerate()
        .map(|(i, player)| {
            (
                u8::try_from(i + 1).unwrap_or(1),
                player.role.display_name().to_string(),
            )
        })
        .collect();
    // 对局 id：绑房间时用 db games.id（观战 ?room= 靠它定位）；裸局用负数自增 id
    let game_id = match request.room_code.as_deref() {
        Some(code) => match crate::routes::rooms::bind_start(&state, code, &seat_roles) {
            Ok((_, game_id)) => game_id,
            Err(err) => {
                tracing::warn!("房间 {code} 绑定失败（按裸局开局，无落库）：{}", err.0);
                state.game.fresh_id()
            }
        },
        None => state.game.fresh_id(),
    };
    let view = state
        .game
        .start(game_id, engine, board_name, seed, seats, {
            // 座位 → Edge 音色：以开局 seed 播种（同局可复现，换局换分配）
            tts::edge::assign(seed, total as usize)
        })
        .map_err(conflict)?;
    // 开局即为本局启动专属自动推进（LLM 决策循环；配置缺失时规则兜底也能推完）
    super::super::orchestr::autoplay::spawn(&state, game_id);
    Ok(ApiResponse::ok(with_mvp(
        &state,
        game_id,
        GameStatusDto::of(&view),
    )))
}

/// 动作类响应（start/pause/resume/end）统一附加 mvp 状态（与 status 轮询同口径）
fn with_mvp(state: &AppState, game_id: i64, mut dto: GameStatusDto) -> GameStatusDto {
    dto.mvp = match dto.ended_reason.as_deref() {
        Some("manual") => None,
        _ => mvp_dto(
            state,
            game_id,
            dto.status == "running" || dto.winner.is_some(),
        ),
    };
    dto
}

/// POST /api/game/pause
#[utoipa::path(
    post,
    path = "/api/game/pause",
    tag = "game",
    operation_id = "pause_game",
    responses(
        (status = 200, description = "已暂停", body = ApiResponse<GameStatusDto>),
        (status = 409, description = "对局不在进行中", body = ApiResponse<String>)
    )
)]
pub async fn pause_game(
    State(state): State<AppState>,
    Query(query): Query<GameQuery>,
) -> ApiResult<Json<ApiResponse<GameStatusDto>>> {
    let game_id = locate_game(&state, &query)?;
    let view = state.game.pause(game_id).map_err(slot_err)?;
    Ok(ApiResponse::ok(with_mvp(
        &state,
        game_id,
        GameStatusDto::of(&view),
    )))
}

/// POST /api/game/resume
#[utoipa::path(
    post,
    path = "/api/game/resume",
    tag = "game",
    operation_id = "resume_game",
    responses(
        (status = 200, description = "已继续", body = ApiResponse<GameStatusDto>),
        (status = 409, description = "对局不在暂停中", body = ApiResponse<String>)
    )
)]
pub async fn resume_game(
    State(state): State<AppState>,
    Query(query): Query<GameQuery>,
) -> ApiResult<Json<ApiResponse<GameStatusDto>>> {
    let game_id = locate_game(&state, &query)?;
    let view = state.game.resume(game_id).map_err(slot_err)?;
    // 恢复后重新拉起本局的自动推进（pause 时循环已退出）
    super::super::orchestr::autoplay::spawn(&state, game_id);
    Ok(ApiResponse::ok(with_mvp(
        &state,
        game_id,
        GameStatusDto::of(&view),
    )))
}

/// POST /api/game/end
#[utoipa::path(
    post,
    path = "/api/game/end",
    tag = "game",
    operation_id = "end_game",
    responses(
        (status = 200, description = "已结束（ended_reason=manual 表示未自然结算的强制终局）", body = ApiResponse<GameStatusDto>),
        (status = 409, description = "没有可结束的对局", body = ApiResponse<String>)
    )
)]
pub async fn end_game(
    State(state): State<AppState>,
    Query(query): Query<GameQuery>,
) -> ApiResult<Json<ApiResponse<GameStatusDto>>> {
    let game_id = locate_game(&state, &query)?;
    let view = state.game.end(game_id).map_err(slot_err)?;
    Ok(ApiResponse::ok(with_mvp(
        &state,
        game_id,
        GameStatusDto::of(&view),
    )))
}

/// GET /api/game/status
#[utoipa::path(
    get,
    path = "/api/game/status",
    tag = "game",
    operation_id = "get_game_status",
    responses(
        (status = 200, description = "当前对局状态与局面总览", body = ApiResponse<GameStatusDto>)
    )
)]
pub async fn get_status(
    State(state): State<AppState>,
    Query(query): Query<GameQuery>,
) -> ApiResult<Json<ApiResponse<GameStatusDto>>> {
    let game_id = locate_game(&state, &query)?;
    Ok(ApiResponse::ok(build_status_dto(&state, game_id)?))
}

/// 组装一局的状态 DTO：槽内实时视图（含事件流 + MVP + 进度落库），
/// 槽内无则从 db 组装 ended 降级结算（服务器重启后的历史局）
fn build_status_dto(state: &AppState, game_id: i64) -> ApiResult<GameStatusDto> {
    if let Ok(view) = state.game.view(game_id) {
        let engine = state
            .game
            .with_engine(game_id, |engine: Option<&GameEngine>| {
                engine.map(|engine| engine.events().to_vec())
            })
            .unwrap_or_default();
        let mut dto = GameStatusDto::of(&view).with_events(engine.as_deref());
        // 手动终局没有 MVP 流程 → null；其余（running/自然结算）按投票状态组装
        dto.mvp = match dto.ended_reason.as_deref() {
            Some("manual") => None,
            _ => mvp_dto(
                state,
                game_id,
                dto.status == "running" || dto.winner.is_some(),
            ),
        };
        // 事件流水增量落库（含对局结束写胜负；幂等，轮询/推送驱动）
        crate::routes::rooms::flush_progress(state, game_id, &dto);
        return Ok(dto);
    }
    stale_status(state, game_id)
}

/// GET /api/game/stream —— SSE 实时事件流（对局变化才推全量快照）。
///
/// 帧体是裸 GameStatusDto（不包 ApiResponse 信封）；对局不可得发 `event: gone` 后关流；
/// 终局（status=ended）推最后一帧后服务端关流，客户端无需重连。
/// 推送间隔 400ms 对比快照签名（事件数 + 状态 + 阶段 + 天数），无变化不发帧；
/// 同时承担事件流水增量落库（前端切到 SSE 后不再轮询，落库不能丢）。
#[utoipa::path(
    get,
    path = "/api/game/stream",
    tag = "game",
    params(GameQuery),
    responses(
        (status = 200, description = "SSE 流：变化帧为裸 GameStatusDto；对局不可得发 event:gone", content_type = "text/event-stream"),
    ),
)]
pub async fn stream_status(
    State(state): State<AppState>,
    Query(query): Query<GameQuery>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, std::convert::Infallible>>> {
    // 定位在握手时做一次；后续对局消失（被 end/清槽）在循环里发 gone
    let located = locate_game(&state, &query).ok();
    let stream = futures_util::stream::unfold(
        StreamState {
            state,
            game_id: located,
            last: None,
            interval: tokio::time::interval(STREAM_POLL),
            done: false,
        },
        |mut st| async move {
            loop {
                if st.done {
                    return None; // gone / 终局帧已发，关流（客户端不重连）
                }
                st.interval.tick().await; // 首个 tick 立即完成，开局帧不等 400ms
                let Some(game_id) = st.game_id else {
                    st.done = true;
                    return Some((Ok(Event::default().event("gone").data("{}")), st));
                };
                let Ok(dto) = build_status_dto(&st.state, game_id) else {
                    // 定位失效 / 槽内无且 db 无 → 对局不可得
                    st.done = true;
                    return Some((Ok(Event::default().event("gone").data("{}")), st));
                };
                // 终局帧：推完即关流（客户端按 onFrame 的 ended 分支结束，不重连）
                if dto.status == "ended" {
                    st.done = true;
                    let frame = serde_json::to_string(&dto).unwrap_or_default();
                    return Some((Ok(Event::default().data(frame)), st));
                }
                // 变化检测：事件条数 + 状态 + 阶段 + 天数足够判变（事件只增不减；
                // 无变化不发帧，空闲保活交给 KeepAlive）
                let signature = (
                    dto.events.len(),
                    dto.status.clone(),
                    dto.phase.clone(),
                    dto.day,
                    dto.alive_total,
                );
                if st.last.as_ref() == Some(&signature) {
                    continue;
                }
                st.last = Some(signature);
                let frame = serde_json::to_string(&dto).unwrap_or_default();
                return Some((Ok(Event::default().data(frame)), st));
            }
        },
    );
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// SSE 推送轮询间隔：autoplay 步进间隔（~2s）的零头，帧延迟感不明显
const STREAM_POLL: std::time::Duration = std::time::Duration::from_millis(400);

/// 快照签名：事件条数 + 状态 + 阶段 + 天数 + 存活数
pub type SnapshotSignature = (usize, String, String, u32, u8);

/// SSE unfold 状态
struct StreamState {
    state: AppState,
    game_id: Option<i64>,
    last: Option<SnapshotSignature>,
    interval: tokio::time::Interval,
    done: bool,
}

/// 组装 status 的 `mvp` 字段（stale 降级局由调用方直接置 null）：
/// - 投票已决出（状态机 Done）→ 从 db 读 MVP 与全部选票，`done`
/// - 投票中（状态机 Pending，或胜负刚落库任务尚未登记）→ `pending`
/// - 无 MVP 流程（投票任务未启动且非 running/自然结算）→ null
fn mvp_dto(state: &AppState, game_id: i64, has_mvp_flow: bool) -> Option<MvpDto> {
    let pending = MvpDto {
        phase: "pending".to_string(),
        winner: None,
        votes: Vec::new(),
    };
    match state.game.mvp_phase(game_id) {
        Some(state::MvpPhase::Pending) => return Some(pending),
        Some(state::MvpPhase::Done) => {}
        // 胜负落库后投票任务异步启动，存在短暂未登记窗口：有 MVP 流程的局按 pending
        None if has_mvp_flow => return Some(pending),
        None => return None,
    }
    let (mvp, votes) =
        crate::routes::rooms::db_query(state, |conn| db::game_mvp_summary(conn, game_id))
            .ok()
            .flatten()?;
    Some(MvpDto {
        phase: "done".to_string(),
        winner: mvp,
        votes: votes
            .into_iter()
            .map(|(voter, candidate, reason)| MvpVoteDto {
                voter,
                candidate,
                reason,
            })
            .collect(),
    })
}

/// 服务器重启后的历史局降级：槽里已无该局，从 db 组装 ended 结算视图。
/// 真实席位身份来自 room_roster，胜负来自 games.winner 列，中断标记与天数
/// 来自 games.result JSON（清扫写入 {"stale":true,...}）。
fn stale_status(state: &AppState, game_id: i64) -> ApiResult<GameStatusDto> {
    let game = crate::routes::rooms::db_query(state, |conn| db::game_result(conn, game_id))?
        .ok_or_else(|| AppError::NotFound("对局不存在或已失效".to_string()))?;
    let roster =
        crate::routes::rooms::db_query(state, |conn| db::room_roster_by_id(conn, game.room_id))?;
    let events = crate::routes::rooms::db_query(state, |conn| db::game_events(conn, game_id))?;
    let result: serde_json::Value = game
        .result
        .as_deref()
        .and_then(|raw| serde_json::from_str(raw).ok())
        .unwrap_or(serde_json::Value::Null);
    // 清扫把 winner 列写成 'stale'：对用户仍是无胜负的中断局
    let stale = game.winner.as_deref() == Some("stale");
    let winner = game.winner.clone().filter(|tag| tag != "stale");
    let settled = winner.is_some();
    let seats: Vec<String> = roster.iter().map(|(_, name, _)| name.clone()).collect();
    let seat_roles: Vec<String> = roster
        .iter()
        .map(|(_, _, role)| role.clone().unwrap_or_else(|| "未知".to_string()))
        .collect();
    let total = u8::try_from(seats.len()).unwrap_or(u8::MAX);
    let day = result
        .get("day")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let dto = GameStatusDto {
        status: "ended".to_string(),
        board: Some(game.board_id),
        seed: 0,
        seat_voices: Vec::new(),
        day: u32::try_from(day).unwrap_or(0),
        phase: "settled".to_string(),
        total,
        alive_total: total,
        roles: Vec::new(),
        winner,
        ended_reason: Some(if stale { "stale" } else { "settled" }.to_string()),
        stale: Some(stale),
        // stale 局无 MVP 投票流程；重启后的已结算历史局从 db 读选票
        mvp: if stale {
            None
        } else {
            mvp_dto(state, game_id, settled)
        },
        seats,
        seat_roles,
        events: events
            .into_iter()
            .map(|event| GameEventDto {
                seq: usize::try_from(event.seq).unwrap_or(0),
                day: event.day,
                kind: event.kind,
                seat: event.seat,
                text: event.content,
                deaths: Vec::new(),
                tie: false,
                target: None,
            })
            .collect(),
    };
    Ok(dto)
}

/// 选手名册（按座位序的默认名单）：来自 state 包的固定 12 人名单
fn werewolf_roster_names() -> Vec<String> {
    state::ROSTER_PLAYERS
        .iter()
        .copied()
        .map(str::to_string)
        .collect()
}

/// GET /api/game/memory 查询参数：玩家座位号（从 0 开始）+ 对局定位
#[derive(Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct MemoryQuery {
    /// 玩家座位号（0 起）
    pub player: u8,
    /// 对局定位参数（room / game，同 [`GameQuery`]）
    #[serde(flatten)]
    pub locate: GameQuery,
}

/// 单条对局记忆的接口形态（通用记忆模型 + 对局领域标签）
#[derive(Serialize, utoipa::ToSchema)]
pub struct MemoryEntryDto {
    /// 对局天（时间片）
    pub seq: u64,
    /// 领域标签：speech / wolfchat / dawn / vote / seercheck / folded
    pub tag: String,
    /// 记录者座位（系统播报为 null）
    pub source: Option<String>,
    /// 事实内容
    pub text: String,
}

/// GET /api/game/memory 响应：某玩家视角的对局记忆（决策 prompt 事实段）
#[derive(Serialize, utoipa::ToSchema)]
pub struct GameMemoryDto {
    /// 记忆主体座位
    pub player: String,
    /// 按天序的事实流水
    pub entries: Vec<MemoryEntryDto>,
    /// 渲染后的可读文本（可直接拼入 LLM prompt）
    pub rendered: String,
}

impl GameMemoryDto {
    fn of(memory: &memory::AgentMemory) -> Self {
        Self {
            player: memory.owner.clone(),
            entries: memory
                .entries
                .iter()
                .map(|entry| MemoryEntryDto {
                    seq: entry.seq,
                    tag: entry.tag.clone(),
                    source: entry.source.clone(),
                    text: entry.text.clone(),
                })
                .collect(),
            rendered: memory.render(),
        }
    }
}

/// GET /api/game/memory：编排层从引擎事件流抽取玩家视角记忆。
/// 若该座位已做过 LLM 压缩，优先返回压缩后的记忆（含摘要行）。
#[utoipa::path(
    get,
    path = "/api/game/memory",
    tag = "game",
    operation_id = "get_game_memory",
    params(MemoryQuery),
    responses(
        (status = 200, description = "玩家视角的对局记忆", body = ApiResponse<GameMemoryDto>),
        (status = 400, description = "座位号越界", body = ApiResponse<String>),
        (status = 409, description = "当前没有进行中的对局", body = ApiResponse<String>)
    )
)]
pub async fn get_memory(
    State(state): State<AppState>,
    Query(query): Query<MemoryQuery>,
) -> ApiResult<Json<ApiResponse<GameMemoryDto>>> {
    let game_id = locate_game(&state, &query.locate)?;
    // 已压缩记忆优先（跨请求存活，含摘要行）；否则从引擎重算
    let stored = state
        .game
        .with_memories(game_id, |memories| memories.get(&query.player).cloned())
        .unwrap_or_default();
    let memory = match stored {
        Some(memory) => memory,
        None => match state.game.with_engine(game_id, |engine| {
            engine
                .ok_or_else(|| AppError::Conflict("当前没有进行中的对局".to_string()))
                .and_then(|engine| {
                    crate::orchestr::memory::recall(engine, query.player)
                        .map_err(|err| AppError::BadRequest(format!("非法座位：{err}")))
                })
        }) {
            Ok(recalled) => recalled?,
            Err(err) => return Err(ApiError(slot_err(err))),
        },
    };
    Ok(ApiResponse::ok(GameMemoryDto::of(&memory)))
}

/// POST /api/game/memory/compact 请求体
#[derive(Deserialize, utoipa::ToSchema)]
pub struct MemoryCompactRequest {
    /// 玩家座位号（0 起）
    pub player: u8,
    /// 保留原文的最近条数（缺省 30）：超过部分交 LLM 压缩为摘要
    pub keep_recent: Option<usize>,
}

/// POST /api/game/memory/compact 响应
#[derive(Serialize, utoipa::ToSchema)]
pub struct MemoryCompactDto {
    /// 本次是否执行了 LLM 压缩（false = 未超窗，无需压缩，模型未被调用）
    pub compacted: bool,
    /// 压缩后的记忆总条数
    pub entries: usize,
    /// LLM 摘要正文（compacted = false 时为 null）
    pub summary: Option<String>,
}

/// AgentError 的映射统一走 `From<AgentError> for AppError`
/// （error/app_error.rs：客户端输入 400，上游/链路 503，其余 500）
/// POST /api/game/memory/compact：记忆过长时调 LLM 压缩老段为摘要。
///
/// 编排链：引擎重算记忆 → [`memory::AgentMemory::compaction_plan`] 划分
/// → LLM 生成摘要 → [`memory::AgentMemory::apply_compaction`] 落回 →
/// 结果在 [`GameState`] 跨请求存活（`GET /api/game/memory` 优先返回）。
/// 未超窗时直接返回 `compacted = false`，不调模型。
#[utoipa::path(
    post,
    path = "/api/game/memory/compact",
    tag = "game",
    operation_id = "compact_game_memory",
    request_body = MemoryCompactRequest,
    responses(
        (status = 200, description = "压缩完成（或无需压缩）", body = ApiResponse<MemoryCompactDto>),
        (status = 400, description = "座位号越界", body = ApiResponse<String>),
        (status = 409, description = "当前没有进行中的对局", body = ApiResponse<String>),
        (status = 500, description = "agent 配置缺失 / 模型返回空摘要 / 压缩应用失败", body = ApiResponse<String>),
        (status = 503, description = "上游模型不可用（网络 / 非 adhere 2xx / 流中断）", body = ApiResponse<String>)
    )
)]
pub async fn compact_memory(
    State(state): State<AppState>,
    query: Query<GameQuery>,
    Json(request): Json<MemoryCompactRequest>,
) -> ApiResult<Json<ApiResponse<MemoryCompactDto>>> {
    let target = GameQuery {
        room: query.room.clone(),
        game: query.game,
    };
    let game_id = locate_game(&state, &target)?;
    // 与 get_memory 一致的缺省窗口：保留最近 30 条原文
    let keep_recent = request.keep_recent.unwrap_or(30);
    let mut memory = match state.game.with_engine(game_id, |engine| {
        engine
            .ok_or_else(|| AppError::Conflict("当前没有进行中的对局".to_string()))
            .and_then(|engine| {
                crate::orchestr::memory::recall(engine, request.player)
                    .map_err(|err| AppError::BadRequest(format!("非法座位：{err}")))
            })
    }) {
        Ok(recalled) => recalled?,
        Err(err) => return Err(ApiError(slot_err(err))),
    };

    // 未超窗：无需压缩，不调模型
    let Some(plan) = memory.compaction_plan(keep_recent) else {
        return Ok(ApiResponse::ok(MemoryCompactDto {
            compacted: false,
            entries: memory.len(),
            summary: None,
        }));
    };

    // LLM 压缩：单轮流式调用，拼接文本增量
    let provider = state.agent.provider();
    let chat = agent::ChatRequest {
        messages: crate::orchestr::memory::compaction_messages(&plan),
        ..agent::ChatRequest::default()
    };
    let mut stream = provider.chat_stream(chat).await?;
    let mut summary = String::new();
    while let Some(event) = stream.next_event().await? {
        if let agent::event::AgentEvent::TextDelta { delta } = event {
            summary.push_str(&delta);
        }
    }
    let summary = summary.trim().to_string();
    if summary.is_empty() {
        return Err(AppError::Internal("模型返回了空摘要".into()).into());
    }

    // 落回 + 持久化（get_memory 优先取这份）
    if !memory.apply_compaction(&plan, summary) {
        return Err(AppError::Internal("记忆压缩应用失败".into()).into());
    }
    let entries = memory.len();
    let summary = memory.entries[0].text.clone();
    state
        .game
        .with_memories(game_id, |memories| memories.insert(request.player, memory))
        .map_err(slot_err)?;
    Ok(ApiResponse::ok(MemoryCompactDto {
        compacted: true,
        entries,
        summary: Some(summary),
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/game/start", post(start_game))
        .route("/api/game/pause", post(pause_game))
        .route("/api/game/resume", post(resume_game))
        .route("/api/game/end", post(end_game))
        .route("/api/game/status", get(get_status))
        .route("/api/game/stream", get(stream_status))
        .route("/api/game/memory", get(get_memory))
        .route("/api/game/memory/compact", post(compact_memory))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
