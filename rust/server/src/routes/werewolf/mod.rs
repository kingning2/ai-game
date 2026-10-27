//! 对局控制面端点：开始 / 暂停 / 继续 / 结束 / 状态查询 / 玩家记忆查询。
//!
//! - `POST /api/game/start`：开局（板子预设 + 可选 seed，可复现）
//! - `POST /api/game/pause`：暂停（引擎保留）
//! - `POST /api/game/resume`：暂停后继续
//! - `POST /api/game/end`：直接结束（强制终局）
//! - `GET  /api/game/status`：当前对局状态与局面总览
//! - `GET  /api/game/memory?player=N`：N 号视角的对局记忆（编排层抽取，供 LLM 决策 prompt）
//! - `POST /api/game/memory/compact`：记忆过长时调 LLM 压缩老段为摘要
//!   （划分 → LLM → 落回，结果跨请求存活；未超窗不调模型）
//!
//! 生命周期非法转移返回 409（如 running 时再 start、idle 时 pause）。
//! Apifox：`GET /api-docs/openapi.json` 的 `game` tag 已含全部端点。

use crate::error::{ApiResponse, ApiResult, AppError};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use state::{AppState, GameState, GameStatus, GameView, InvalidTransition};
use werewolf::{Action, DeathCause, GameEngine, GameEvent, Outcome, PlayerId, Role};

/// POST /api/game/start 请求体：全字段可选（缺省 = 全明星板 + 随机种子）
#[derive(Default, Deserialize, utoipa::ToSchema)]
pub struct GameStartRequest {
    /// 板子预设：`newbie_six`（新手 6 人局）/ `nine_standard`（经典 9 人局，缺省）/ `all_stars`（全角色 14 人局）
    pub board: Option<String>,
    /// 随机种子：缺省或 0 = 服务端随机生成；同种子同发牌（可复现）
    pub seed: Option<u64>,
    /// 座位 → 选手名（长度须等于座位数；缺省取选手名册前 N 位）。
    /// LLM 决策按名取选手模型配置
    pub players: Option<Vec<String>>,
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
}

/// 上帝视角事件渲染：`(day, kind, seat, 文本)`；未知事件返回 `None`
pub fn render_event(
    event: &GameEvent,
    secret_visible: bool,
) -> Option<(u32, &'static str, Option<PlayerId>, String)> {
    #[allow(clippy::type_complexity)]
    fn inner(
        event: &GameEvent,
        secret_visible: bool,
    ) -> Option<(u32, &'static str, Option<PlayerId>, String)> {
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
            GameEvent::NightBegan { day } => {
                Some((*day, "night", None, format!("第 {day} 夜天黑")))
            }
            GameEvent::NightAction { actor, action } => secret_visible.then(|| {
                (
                    0,
                    "night_action",
                    Some(actor.id),
                    format!(
                        "{}号（{}）：{}",
                        actor.id,
                        actor.role.display_name(),
                        action_text(action)
                    ),
                )
            }),
            GameEvent::WolfChatSpoke { day, speaker, text } => secret_visible.then(|| {
                (
                    *day,
                    "wolf_chat",
                    Some(speaker.id),
                    format!("{}号（狼）耳语：{text}", speaker.id),
                )
            }),
            GameEvent::WolfChatPassed { day, speaker } => secret_visible.then(|| {
                (
                    *day,
                    "wolf_pass",
                    Some(speaker.id),
                    format!("{}号（狼）保持沉默", speaker.id),
                )
            }),
            GameEvent::Dawn { day, deaths } => {
                let text = if deaths.is_empty() {
                    "平安夜".to_string()
                } else {
                    let names = deaths
                        .iter()
                        .map(|d| format!("{}号（{}）", d.player, death_text(&d.cause)))
                        .collect::<Vec<_>>()
                        .join("、");
                    format!("昨夜死亡：{names}")
                };
                Some((*day, "dawn", None, text))
            }
            GameEvent::SpeechMade { day, speaker, text } => {
                Some((*day, "speech", Some(*speaker), text.clone()))
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
                Some(*knight),
                format!(
                    "{knight}号决斗{target}号：{}（{}是狼人）出局",
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
                    .map(|(seat, n)| format!("{seat}号x{n}"))
                    .collect::<Vec<_>>()
                    .join("、");
                let text = if *tie {
                    format!("平票，无人放逐（{votes}）")
                } else {
                    format!(
                        "放逐{}号（{}）；{votes}",
                        exiled.as_ref().map(|p| p.id).unwrap_or_default(),
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
                Some(*idiot),
                format!("{idiot}号白痴翻牌免死，失去投票权"),
            )),
            GameEvent::HunterShotFired {
                day,
                hunter,
                target,
            } => Some((
                *day,
                "hunter",
                Some(hunter.id),
                match target {
                    Some(t) => format!(
                        "{}号（{}）开枪带走{}号（{}）",
                        hunter.id,
                        hunter.role.display_name(),
                        t.id,
                        t.role.display_name()
                    ),
                    None => format!("{}号（{}）放弃开枪", hunter.id, hunter.role.display_name()),
                },
            )),
            GameEvent::PlayerDied { day, player, cause } => Some((
                *day,
                "died",
                Some(player.id),
                format!(
                    "{}号（{}）出局：{}",
                    player.id,
                    player.role.display_name(),
                    death_text(cause)
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
    inner(event, secret_visible)
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
fn death_text(cause: &DeathCause) -> &'static str {
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
            events: Vec::new(),
        }
    }

    /// 附加上帝视角事件流（开局后才有引擎可取）
    pub fn with_events(mut self, engine: Option<&GameEngine>) -> Self {
        if let Some(engine) = engine {
            self.events = engine
                .events()
                .iter()
                .enumerate()
                .filter_map(|(seq, event)| {
                    render_event(event, true).map(|(day, kind, seat, text)| GameEventDto {
                        seq,
                        day,
                        kind: kind.to_string(),
                        seat,
                        text,
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
    let view = state
        .game
        .start(engine, board_name, seed, seats)
        .map_err(conflict)?;
    // 开局即启动自动推进（LLM 决策循环；配置缺失时规则兜底也能推完）
    super::super::orchestr::autoplay::spawn(&state);
    Ok(ApiResponse::ok(GameStatusDto::of(&view)))
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
) -> ApiResult<Json<ApiResponse<GameStatusDto>>> {
    let view = state.game.pause().map_err(conflict)?;
    Ok(ApiResponse::ok(GameStatusDto::of(&view)))
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
) -> ApiResult<Json<ApiResponse<GameStatusDto>>> {
    let view = state.game.resume().map_err(conflict)?;
    // 恢复后重新拉起自动推进（pause 时循环已退出）
    super::super::orchestr::autoplay::spawn(&state);
    Ok(ApiResponse::ok(GameStatusDto::of(&view)))
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
) -> ApiResult<Json<ApiResponse<GameStatusDto>>> {
    let view = state.game.end().map_err(conflict)?;
    Ok(ApiResponse::ok(GameStatusDto::of(&view)))
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
) -> ApiResult<Json<ApiResponse<GameStatusDto>>> {
    let view = state.game.view();
    let engine = state.game.with_engine(|engine| engine.cloned());
    Ok(ApiResponse::ok(
        GameStatusDto::of(&view).with_events(engine.as_ref()),
    ))
}

/// 选手名册（按座位序的默认名单）：来自 state 包的固定 12 人名单
fn werewolf_roster_names() -> Vec<String> {
    state::ROSTER_PLAYERS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// GET /api/game/memory 查询参数：玩家座位号（从 0 开始）
#[derive(Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct MemoryQuery {
    /// 玩家座位号（0 起）
    pub player: u8,
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
    // 已压缩记忆优先（跨请求存活，含摘要行）；否则从引擎重算
    let stored = state
        .game
        .with_memories(|memories| memories.get(&query.player).cloned());
    let memory = match stored {
        Some(memory) => memory,
        None => state.game.with_engine(|engine| {
            let engine =
                engine.ok_or_else(|| AppError::Conflict("当前没有进行中的对局".to_string()))?;
            crate::orchestr::memory::recall(engine, query.player)
                .map_err(|err| AppError::BadRequest(format!("非法座位：{err}")))
        })?,
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

/// AgentError → AppError：配置/请求问题 400，上游与链路问题 503
fn agent_error(err: agent::AgentError) -> AppError {
    match err {
        agent::AgentError::Config(msg) | agent::AgentError::InvalidRequest(msg) => {
            AppError::BadRequest(msg)
        }
        _ => AppError::ServiceUnavailable,
    }
}

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
        (status = 400, description = "座位号越界 / agent 未配置 API Key", body = ApiResponse<String>),
        (status = 409, description = "当前没有进行中的对局", body = ApiResponse<String>),
        (status = 500, description = "模型返回空摘要或压缩应用失败", body = ApiResponse<String>),
        (status = 503, description = "上游模型不可用（网络 / 非 adhere 2xx / 流中断）", body = ApiResponse<String>)
    )
)]
pub async fn compact_memory(
    State(state): State<AppState>,
    Json(request): Json<MemoryCompactRequest>,
) -> ApiResult<Json<ApiResponse<MemoryCompactDto>>> {
    // 与 get_memory 一致的缺省窗口：保留最近 30 条原文
    let keep_recent = request.keep_recent.unwrap_or(30);
    let mut memory = state.game.with_engine(|engine| {
        let engine =
            engine.ok_or_else(|| AppError::Conflict("当前没有进行中的对局".to_string()))?;
        crate::orchestr::memory::recall(engine, request.player)
            .map_err(|err| AppError::BadRequest(format!("非法座位：{err}")))
    })?;

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
        model: None,
        messages: crate::orchestr::memory::compaction_messages(&plan),
        thinking: None,
    };
    let mut stream = provider.chat_stream(chat).await.map_err(agent_error)?;
    let mut summary = String::new();
    while let Some(event) = stream.next_event().await.map_err(agent_error)? {
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
        .with_memories(|memories| memories.insert(request.player, memory));
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
        .route("/api/game/memory", get(get_memory))
        .route("/api/game/memory/compact", post(compact_memory))
}

#[cfg(test)]
mod tests {
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

        // 初始状态：idle
        let (status, body) = send(app, "GET", "/api/game/status", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"]["status"], "idle");
        assert_eq!(body["data"]["phase"], "idle");

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

        // 进行中重复开局 → 409
        let app = Router::new().merge(router()).with_state(state.clone());
        let (status, body) = send(app, "POST", "/api/game/start", Some(json!({}))).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], 10005);

        // 暂停 → 继续
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
        assert_eq!(status, StatusCode::CONFLICT, "未开局应 409");
        assert_eq!(body["code"], 10005);
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
        assert_eq!(status, StatusCode::CONFLICT, "未开局应 409");
        assert_eq!(body["code"], 10005);
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
}
