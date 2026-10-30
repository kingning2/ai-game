//! 狼人杀引擎门面 —— [`GameEngine`] 现在是 [`game_runtime::GameManager`] 的
//! 逐字转发壳（Bevy ECS 迁移收尾）。
//!
//! 历史：这里曾是 873 行手写状态机（阶段 1-3 审计后按 spec §12 逐段迁入
//! game-ecs System）；现在全部行为由 ECS 运行时承担，黄金对拍测试守护等价性。
//! 对外 API 与旧引擎完全一致：server / agent / 快照层零改动。

use game_domain::{Action, BoardConfig, BoardError, GameError, Phase, PlayerId, PlayerState};
use game_events::GameEvent;
use game_runtime::GameManager;

/// 狼人杀游戏引擎：纯状态机，零 IO。
///
/// 推进方式：调用方读 [`GameEngine::pending`] 了解当前等谁做什么，
/// 喂入合法 [`Action`]；引擎校验后推进状态机并返回增量事件流。
/// 对外可见的保密视图见 [`crate::snapshot`]（身份只在上帝视角/翻牌后可见）。
#[derive(Debug)]
pub struct GameEngine(GameManager);

impl GameEngine {
    /// 开局：校验板子、洗牌发牌。`seed` 相同 ⇒ 发牌相同（可复现）
    pub fn new(board: BoardConfig, seed: u64) -> Result<Self, BoardError> {
        Ok(Self(GameManager::new(board, seed)?))
    }

    /// 当前阶段
    pub fn phase(&self) -> &Phase {
        self.0.phase()
    }

    /// 完整上帝事件日志
    pub fn events(&self) -> &[GameEvent] {
        self.0.events()
    }

    pub fn settled(&self) -> bool {
        self.0.settled()
    }

    /// 座位数
    pub fn total_players(&self) -> u8 {
        self.0.total_players()
    }

    /// 玩家状态（按座位序；保密靠快照层裁剪）
    pub fn players(&self) -> &[PlayerState] {
        self.0.players()
    }

    /// 当前等谁做什么
    pub fn pending(&self) -> Option<(PlayerId, &'static str)> {
        self.0.pending()
    }

    /// 喂入一个动作，推进状态机，返回本次产生的增量事件
    pub fn apply(&mut self, actor: PlayerId, action: Action) -> Result<Vec<GameEvent>, GameError> {
        self.0.apply(actor, action)
    }
}
