//! 对局控制面：生命周期状态机 + 引擎句柄。
//!
//! 生命周期（本模块）与局内推进（game crate 的 Phase）是两层：
//! - 这里管「这局开没开 / 暂没暂停 / 结没结束」：`Idle → Running ⇄ Paused → Ended`
//! - 引擎管「局内走到第几夜第几票」；本模块只读引擎派生总览，不喂动作
//! - 结束只做控制面标记：引擎自然分出胜负（`Settled`）与手动 `end` 都落到 `Ended`，
//!   由 `GameView::outcome` 是否有值区分（`settled` / `manual`）
//!
//! AI 发言驱动（调 `GameEngine::apply`）属于下一环，由 server 编排后接入。

use std::collections::HashMap;
use std::fmt;
use std::sync::RwLock;

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

struct Inner {
    status: GameStatus,
    engine: Option<GameEngine>,
    board: Option<String>,
    seed: u64,
    /// 座位 → 选手名（LLM 决策时按名取模型配置）
    seats: Vec<String>,
    /// 压缩记忆表（座位 → 已应用 LLM 摘要的记忆）：
    /// `recall` 重算不含摘要，压缩结果在这里跨请求存活；开局清空
    memories: HashMap<u8, AgentMemory>,
}

impl Inner {
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
        }
    }
}

/// 对局运行时：全局唯一「当前对局」（单局制；多局管理有需求再扩成表）
///
/// 锁中毒按本 crate 惯例放弃守卫不 panic：`unwrap_or_else(PoisonError::into_inner)`。
pub struct GameState {
    inner: RwLock<Inner>,
}

impl fmt::Debug for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f.debug_struct("GameState")
            .field("status", &inner.status)
            .field("board", &inner.board)
            .field("seed", &inner.seed)
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
            inner: RwLock::new(Inner {
                status: GameStatus::Idle,
                engine: None,
                board: None,
                seed: 0,
                seats: Vec::new(),
                memories: HashMap::new(),
            }),
        }
    }

    /// 当前视图
    pub fn view(&self) -> GameView {
        self.inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot()
    }

    /// 锁内借用当前引擎执行 `f`（未开局为 `None`）。
    ///
    /// 引擎不跨锁返回引用，结果值由 `f` 产出——编排层
    /// （server）用它对进行中的对局做只读分析。
    pub fn with_engine<R>(&self, f: impl FnOnce(Option<&GameEngine>) -> R) -> R {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(inner.engine.as_ref())
    }

    /// 锁内可变借用当前引擎执行 `f`（未开局为 `None`）——编排层喂入动作用。
    pub fn with_engine_mut<R>(&self, f: impl FnOnce(Option<&mut GameEngine>) -> R) -> R {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(inner.engine.as_mut())
    }

    /// 锁内读写压缩记忆表（座位 → 已应用 LLM 摘要的记忆）。
    ///
    /// 压缩由 server 编排（`POST /api/game/memory/compact`：
    /// 划分 → LLM 摘要 → 落回），这里只负责跨请求存活；开局清空。
    pub fn with_memories<R>(&self, f: impl FnOnce(&mut HashMap<u8, AgentMemory>) -> R) -> R {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(&mut inner.memories)
    }

    /// 开局：引擎由调用方构造好传入（板子校验错误留在 server 层映射 400）。
    /// `seats` 为座位 → 选手名（长度须等于座位数，LLM 决策按名取模型配置）。
    /// `Idle` 或 `Ended` 才能开局；进行中 / 暂停中返回 409 语义错误。
    pub fn start(
        &self,
        engine: GameEngine,
        board_name: String,
        seed: u64,
        seats: Vec<String>,
    ) -> Result<GameView, InvalidTransition> {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(inner.status, GameStatus::Idle | GameStatus::Ended) {
            return Err(InvalidTransition {
                action: "开局",
                from: inner.status,
            });
        }
        inner.status = GameStatus::Running;
        inner.engine = Some(engine);
        inner.board = Some(board_name);
        inner.seed = seed;
        inner.seats = seats;
        inner.memories.clear();
        Ok(inner.snapshot())
    }

    /// 暂停：仅 `Running` 可暂停
    pub fn pause(&self) -> Result<GameView, InvalidTransition> {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if inner.status != GameStatus::Running {
            return Err(InvalidTransition {
                action: "暂停",
                from: inner.status,
            });
        }
        inner.status = GameStatus::Paused;
        Ok(inner.snapshot())
    }

    /// 继续：仅 `Paused` 可继续
    pub fn resume(&self) -> Result<GameView, InvalidTransition> {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if inner.status != GameStatus::Paused {
            return Err(InvalidTransition {
                action: "继续",
                from: inner.status,
            });
        }
        inner.status = GameStatus::Running;
        Ok(inner.snapshot())
    }

    /// 结束：`Running` / `Paused` 可结束（强制终局，`outcome` 为空 = 手动结束）
    pub fn end(&self) -> Result<GameView, InvalidTransition> {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(inner.status, GameStatus::Running | GameStatus::Paused) {
            return Err(InvalidTransition {
                action: "结束",
                from: inner.status,
            });
        }
        inner.status = GameStatus::Ended;
        Ok(inner.snapshot())
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
mod tests {
    use super::*;
    use werewolf::BoardError;

    fn start_game(state: &GameState, seed: u64) -> Result<GameView, InvalidTransition> {
        let engine = GameEngine::new(BoardConfig::nine_standard(), seed)
            .map_err(|err: BoardError| panic!("预设板子应恒合法：{err}"));
        state.start(
            engine.expect("预设板子应恒合法"),
            "nine_standard".into(),
            seed,
            vec!["选手".to_string(); 9],
        )
    }

    #[test]
    fn full_lifecycle() {
        let state = GameState::new();
        assert_eq!(state.view().status, GameStatus::Idle);
        assert_eq!(state.view().phase, "idle");

        let view = start_game(&state, 7).expect("idle 可开局");
        assert_eq!(view.status, GameStatus::Running);
        assert_eq!(view.phase, "night");
        assert_eq!(view.board.as_deref(), Some("nine_standard"));
        assert_eq!(view.seed, 7);
        let summary = view.summary.expect("开局后应有总览");
        assert_eq!(summary.total, 9);
        assert_eq!(summary.alive_total, 9);

        state.pause().expect("running 可暂停");
        assert_eq!(state.view().status, GameStatus::Paused);
        state.resume().expect("paused 可继续");
        assert_eq!(state.view().status, GameStatus::Running);

        let view = state.end().expect("running 可结束");
        assert_eq!(view.status, GameStatus::Ended);
        assert!(view.outcome.is_none(), "手动结束未结算，outcome 应为空");
    }

    #[test]
    fn invalid_transitions_rejected() {
        let state = GameState::new();

        let err = state.pause().expect_err("idle 不能暂停");
        assert_eq!(err.from, GameStatus::Idle);
        let err = state.resume().expect_err("idle 不能继续");
        assert_eq!(err.action, "继续");
        let err = state.end().expect_err("idle 不能结束");
        assert_eq!(err.from, GameStatus::Idle);

        start_game(&state, 1).expect("开局");
        let err = state
            .start(
                GameEngine::new(BoardConfig::nine_standard(), 2).expect("合法"),
                "nine_standard".into(),
                2,
                vec!["选手".to_string(); 9],
            )
            .expect_err("running 不能重复开局");
        assert_eq!(err.from, GameStatus::Running);

        state.pause().expect("暂停");
        let err = state.pause().expect_err("paused 不能再暂停");
        assert_eq!(err.from, GameStatus::Paused);
        let err = state
            .start(
                GameEngine::new(BoardConfig::nine_standard(), 2).expect("合法"),
                "nine_standard".into(),
                2,
                vec!["选手".to_string(); 9],
            )
            .expect_err("paused 不能开局");
        assert_eq!(err.from, GameStatus::Paused);

        let view = state.end().expect("paused 可结束");
        assert_eq!(view.status, GameStatus::Ended);
        let err = state.end().expect_err("ended 不能再结束");
        assert_eq!(err.from, GameStatus::Ended);
    }

    #[test]
    fn restart_after_end() {
        let state = GameState::new();
        start_game(&state, 3).expect("第一局");
        state.end().expect("结束");
        assert_eq!(state.view().status, GameStatus::Ended);

        start_game(&state, 4).expect("ended 可重新开局");
        let view = state.view();
        assert_eq!(view.status, GameStatus::Running);
        assert_eq!(view.seed, 4, "新局覆盖旧局元数据");
        assert_eq!(view.summary.expect("新局总览").alive_total, 9);
    }

    #[test]
    fn resolve_board_unknown_name() {
        assert!(GameState::resolve_board("nine_standard").is_some());
        assert!(GameState::resolve_board("all_stars").is_some());
        assert!(GameState::resolve_board("newbie_six").is_some());
        assert!(GameState::resolve_board("nope").is_none());
    }
}
