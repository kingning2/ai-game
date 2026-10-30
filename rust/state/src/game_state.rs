//! 对局控制面：生命周期状态机 + 引擎句柄（多槽位：按对局 id 索引）。
//!
//! 生命周期（本模块）与局内推进（game crate 的 Phase）是两层：
//! - 这里管「这局开没开 / 暂没暂停 / 结没结束」：`Idle → Running ⇄ Paused → Ended`
//! - 引擎管「局内走到第几夜第几票」；本模块只读引擎派生总览，不喂动作
//! - 结束只做控制面标记：引擎自然分出胜负（`Settled`）与手动 `end` 都落到 `Ended`，
//!   由 `GameView::outcome` 是否有值区分（`settled` / `manual`）
//!
//! 多槽位：每个 `game_id`（SQLite `games.id`；不落库的裸局用负数自增 id）一个独立
//! 槽位，多房间并发对局互不串局；`latest` 记录最近启动的对局 id，兼容无参调用。
//!
//! 锁纪律：槽位表用 `RwLock`（短临界区，只克隆 `Arc`），单局状态用 `std::sync::Mutex`
//!（闭包内不得 `.await`）；锁中毒按本 crate 惯例放弃守卫不 panic：
//! `unwrap_or_else(PoisonError::into_inner)`。

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use memory::AgentMemory;
use werewolf::{BoardConfig, GameEngine, GameSummary, Outcome, Phase};

/// 对局生命周期（服务层控制面，与引擎内局面无关）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GameStatus {
    /// 未开局（服务启动后的初始态；或上一局已结束被重新开局前）
    #[default]
    Idle,
    /// 进行中
    Running,
    /// 暂停（引擎保留，恢复后继续）
    Paused,
    /// 已结束（自然结算或手动结束）
    Ended,
}

/// 非法生命周期转移（handler 映射为 409 Conflict）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTransition {
    /// 动作名：`start` / `pause` / `resume` / `end`
    pub action: &'static str,
    /// 转移发生时的状态
    pub from: GameStatus,
}

impl fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "当前对局状态为 {}，不允许 {}",
            self.from.as_str(),
            self.action
        )
    }
}

impl std::error::Error for InvalidTransition {}

/// 对局槽位错误（路由层映射：`Config` → 404，`Transition` → 409）
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GameError {
    /// 对局不存在或已失效（槽里没有这个 game_id，如服务器重启后的历史局）
    #[error("{0}")]
    Config(String),
    /// 非法生命周期转移
    #[error("{0}")]
    Transition(#[from] InvalidTransition),
}

impl GameStatus {
    /// 状态的接口字符串形态（idle / running / paused / ended）
    pub fn as_str(self) -> &'static str {
        match self {
            GameStatus::Idle => "idle",
            GameStatus::Running => "running",
            GameStatus::Paused => "paused",
            GameStatus::Ended => "ended",
        }
    }
}

/// MVP 投票阶段（per-game，仿 slots 模式；选票与结果持久化在 db，这里只存阶段）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MvpPhase {
    /// 投票进行中（胜负落库后、MVP 未决出）
    Pending,
    /// 已决出（`games.mvp` 与 `mvp_votes` 均已落库，票从 db 读）
    Done,
}

impl MvpPhase {
    /// 接口字符串形态（pending / done）
    pub fn as_str(self) -> &'static str {
        match self {
            MvpPhase::Pending => "pending",
            MvpPhase::Done => "done",
        }
    }
}

/// 对局某一刻的控制面视图（锁内派生的只读快照）
#[derive(Debug, Clone)]
pub struct GameView {
    pub status: GameStatus,
    /// 板子预设名（`nine_standard` / `all_stars`）；未开局为 `None`
    pub board: Option<String>,
    /// 开局种子（可复现发牌与验人顺序）
    pub seed: u64,
    /// 局面总览（存活 / 身份统计 / 天数）；未开局为 `None`
    pub summary: Option<GameSummary>,
    /// 引擎内局面阶段文本（night / day_speech / …）；未开局为 `idle`
    pub phase: &'static str,
    /// 引擎已自然分出胜负时的结果；手动结束（未结算）为 `None`
    pub outcome: Option<Outcome>,
    /// 座位 → 选手名（LLM 决策时按名取模型配置；未开局为空）
    pub seats: Vec<String>,
    /// 座位 → 角色显示名（上帝视角；未开局为空）
    pub seat_roles: Vec<String>,
    /// 座位 → Edge TTS 音色（观战语音播报用；未分配/未开局为空）
    pub seat_voices: Vec<String>,
}

/// 引擎局面阶段的接口文本
fn phase_text(engine: &GameEngine) -> &'static str {
    match engine.phase() {
        Phase::Night { .. } => "night",
        Phase::DaySpeech { .. } => "day_speech",
        Phase::KnightDuel { .. } => "knight_duel",
        Phase::DayVote { .. } => "day_vote",
        Phase::HunterShot { .. } => "hunter_shot",
        Phase::Settled { .. } => "settled",
    }
}

/// 单局运行时槽位（一局的全部控制面状态；房间绑定关系由 db 承担，不再挂在槽上）
struct Slot {
    status: GameStatus,
    engine: Option<GameEngine>,
    board: Option<String>,
    seed: u64,
    /// 座位 → 选手名（LLM 决策时按名取模型配置）
    seats: Vec<String>,
    /// 座位 → Edge TTS 音色（观战语音播报用）
    seat_voices: Vec<String>,
    /// 压缩记忆表（座位 → 已应用 LLM 摘要的记忆）：
    /// `recall` 重算不含摘要，压缩结果在这里跨请求存活；开局清空
    memories: HashMap<u8, AgentMemory>,
}

impl Slot {
    fn new() -> Self {
        Self {
            status: GameStatus::Idle,
            engine: None,
            board: None,
            seed: 0,
            seats: Vec::new(),
            seat_voices: Vec::new(),
            memories: HashMap::new(),
        }
    }

    /// 锁内派生只读视图（避免嵌套取锁）
    fn snapshot(&self) -> GameView {
        GameView {
            status: self.status,
            board: self.board.clone(),
            seed: self.seed,
            summary: self.engine.as_ref().map(GameSummary::of),
            phase: self
                .engine
                .as_ref()
                .map_or("idle", |engine| phase_text(engine)),
            outcome: self
                .engine
                .as_ref()
                .and_then(|engine| match engine.phase() {
                    Phase::Settled { outcome } => Some(outcome.clone()),
                    _ => None,
                }),
            seats: self.seats.clone(),
            seat_voices: self.seat_voices.clone(),
            // 上帝视角：座位 → 角色显示名（观战/发牌页与引擎真实身份对齐）
            seat_roles: self
                .engine
                .as_ref()
                .map(|engine| {
                    engine
                        .players()
                        .iter()
                        .map(|p| p.role.display_name().to_string())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

/// 对局运行时：多槽位管理器，按 `game_id` 索引各局。
///
/// `latest` 记录最近启动的 `game_id`（`start` 成功即更新），
/// 供无定位参数的路由兜底（HashMap 迭代无序，latest 是唯一的确定性入口）。
pub struct GameState {
    /// game_id → 槽位（`Arc` 外提：定位与单局锁解耦，持 `Arc` 期间槽位不会消失）
    slots: RwLock<HashMap<i64, Arc<Mutex<Slot>>>>,
    /// game_id → MVP 投票阶段（仿 slots；同局重新开局时重置）
    mvp: RwLock<HashMap<i64, MvpPhase>>,
    /// 最近启动的对局 id
    latest: RwLock<Option<i64>>,
    /// 裸局（不落库）的自增 id：负数递减，永不与 db 正数 id 冲突
    bare_ids: AtomicI64,
}

impl fmt::Debug for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slots = self
            .slots
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut ids: Vec<i64> = slots.keys().copied().collect();
        ids.sort_unstable();
        f.debug_struct("GameState")
            .field("slot_ids", &ids)
            .field("latest", &self.latest())
            .finish()
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        Self {
            slots: RwLock::new(HashMap::new()),
            mvp: RwLock::new(HashMap::new()),
            latest: RwLock::new(None),
            bare_ids: AtomicI64::new(0),
        }
    }

    /// 裸局 id：负数自减（-1, -2, …），永不与 db 正数 id 冲突
    pub fn fresh_id(&self) -> i64 {
        let prev = self.bare_ids.fetch_sub(1, Ordering::Relaxed);
        prev - 1
    }

    /// 最近启动的对局 id（无则 `None`）
    pub fn latest(&self) -> Option<i64> {
        *self
            .latest
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// MVP 投票阶段（无记录 = 本局未触发投票或已清理）
    pub fn mvp_phase(&self, game_id: i64) -> Option<MvpPhase> {
        Some(
            *self
                .mvp
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&game_id)?,
        )
    }

    /// 标记 MVP 投票阶段（编排层在胜负落库后置 Pending，决出后置 Done）
    pub fn set_mvp_phase(&self, game_id: i64, phase: MvpPhase) {
        self.mvp
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(game_id, phase);
    }

    /// 定位槽位：不存在返回 `Config` 错误（路由层映射 404）
    fn slot(&self, game_id: i64) -> Result<Arc<Mutex<Slot>>, GameError> {
        self.slots
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&game_id)
            .cloned()
            .ok_or_else(|| GameError::Config("对局不存在或已失效".to_string()))
    }

    /// 当前视图
    pub fn view(&self, game_id: i64) -> Result<GameView, GameError> {
        let slot = self.slot(game_id)?;
        Ok(slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot())
    }

    /// 锁内借用当前引擎执行 `f`（未开局为 `None`）。
    ///
    /// 引擎不跨锁返回引用，结果值由 `f` 产出——编排层
    /// （server）用它对进行中的对局做只读分析。
    pub fn with_engine<R>(
        &self,
        game_id: i64,
        f: impl FnOnce(Option<&GameEngine>) -> R,
    ) -> Result<R, GameError> {
        let slot = self.slot(game_id)?;
        let guard = slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(f(guard.engine.as_ref()))
    }

    /// 锁内可变借用当前引擎执行 `f`（未开局为 `None`）——编排层喂入动作用。
    pub fn with_engine_mut<R>(
        &self,
        game_id: i64,
        f: impl FnOnce(Option<&mut GameEngine>) -> R,
    ) -> Result<R, GameError> {
        let slot = self.slot(game_id)?;
        let mut guard = slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(f(guard.engine.as_mut()))
    }

    /// 锁内读写压缩记忆表（座位 → 已应用 LLM 摘要的记忆）。
    ///
    /// 压缩由 server 编排（`POST /api/game/memory/compact`：
    /// 划分 → LLM 摘要 → 落回），这里只负责跨请求存活；开局清空。
    pub fn with_memories<R>(
        &self,
        game_id: i64,
        f: impl FnOnce(&mut HashMap<u8, AgentMemory>) -> R,
    ) -> Result<R, GameError> {
        let slot = self.slot(game_id)?;
        let mut guard = slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(f(&mut guard.memories))
    }

    /// 开局：引擎由调用方构造好传入（板子校验错误留在 server 层映射 400）。
    /// `seats` 为座位 → 选手名（长度须等于座位数，LLM 决策按名取模型配置）。
    /// 槽位已存在且进行中 / 暂停中返回 409 语义错误（正常情况下每次开局
    /// 都是全新 game_id，同 id 重复 start 属调用方 bug）。
    /// 成功后本局成为 `latest`。
    pub fn start(
        &self,
        game_id: i64,
        engine: GameEngine,
        board_name: String,
        seed: u64,
        seats: Vec<String>,
        seat_voices: Vec<String>,
    ) -> Result<GameView, InvalidTransition> {
        let mut slot_map = self
            .slots
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let slot = slot_map
            .entry(game_id)
            .or_insert_with(|| Arc::new(Mutex::new(Slot::new())));
        let mut guard = slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(guard.status, GameStatus::Idle | GameStatus::Ended) {
            return Err(InvalidTransition {
                action: "开局",
                from: guard.status,
            });
        }
        guard.status = GameStatus::Running;
        guard.engine = Some(engine);
        guard.board = Some(board_name);
        guard.seed = seed;
        guard.seats = seats;
        guard.seat_voices = seat_voices;
        guard.memories.clear();
        drop(guard);
        drop(slot_map);
        // 同 id 重新开局：重置上一局的 MVP 投票阶段（状态清理；end 后保留
        // 是因为投票任务在局结束后才跑完，此处才是真正可清理的时点）
        self.mvp
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&game_id);
        *self
            .latest
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(game_id);
        // 重新取锁派生快照（上面已释放，避免持双锁跨方法）
        self.view(game_id).map_err(|err| match err {
            GameError::Transition(t) => t,
            GameError::Config(_) => InvalidTransition {
                action: "开局",
                from: GameStatus::Idle,
            },
        })
    }

    /// 暂停：仅 `Running` 可暂停
    pub fn pause(&self, game_id: i64) -> Result<GameView, GameError> {
        let slot = self.slot(game_id)?;
        let mut guard = slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.status != GameStatus::Running {
            return Err(GameError::Transition(InvalidTransition {
                action: "暂停",
                from: guard.status,
            }));
        }
        guard.status = GameStatus::Paused;
        Ok(guard.snapshot())
    }

    /// 继续：仅 `Paused` 可继续
    pub fn resume(&self, game_id: i64) -> Result<GameView, GameError> {
        let slot = self.slot(game_id)?;
        let mut guard = slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.status != GameStatus::Paused {
            return Err(GameError::Transition(InvalidTransition {
                action: "继续",
                from: guard.status,
            }));
        }
        guard.status = GameStatus::Running;
        Ok(guard.snapshot())
    }

    /// 结束：`Running` / `Paused` 可结束（强制终局，`outcome` 为空 = 手动结束）
    pub fn end(&self, game_id: i64) -> Result<GameView, GameError> {
        let slot = self.slot(game_id)?;
        let mut guard = slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(guard.status, GameStatus::Running | GameStatus::Paused) {
            return Err(GameError::Transition(InvalidTransition {
                action: "结束",
                from: guard.status,
            }));
        }
        guard.status = GameStatus::Ended;
        Ok(guard.snapshot())
    }

    /// 板子预设解析：接口层只传名字，这里映射到配置（未知名字返回 `None`）
    pub fn resolve_board(name: &str) -> Option<BoardConfig> {
        match name {
            "newbie_six" => Some(BoardConfig::newbie_six()),
            "nine_standard" => Some(BoardConfig::nine_standard()),
            "all_stars" => Some(BoardConfig::all_stars()),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "game_state_tests.rs"]
mod tests;
