//! 资源定义：游戏阶段 / 事件日志 / RNG / 女巫药水 / 开局配置。

use bevy_ecs::prelude::*;
use game_domain::{BoardConfig, Phase, Rng};
use game_events::GameEvent;

/// 游戏阶段（迁移期唯一写入者是开局装配；后续归 PhaseTransitionSystem 独占）
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct GamePhase(pub Phase);

/// 事件日志：唯一事实源（seq = 下标，单调连续；SSE / 前端去重契约依赖它）
#[derive(Resource, Debug, Default)]
pub struct EventLog {
    pub events: Vec<GameEvent>,
}

/// 确定性 RNG（splitmix64 + Fisher–Yates，`game_domain::Rng` 的资源包装；
/// seed 相同 ⇒ 发牌相同，可复现测试）
#[derive(Resource, Debug, Clone)]
pub struct GameRng(pub Rng);

/// 女巫药水持有量（各一瓶，整局有效；引擎里全局唯一 ⇒ Resource 而非组件）
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Potions {
    pub save: bool,
    pub poison: bool,
}

/// 开局配置（Setup 阶段消费一次，发牌后不再使用）
#[derive(Resource, Debug, Clone)]
pub struct SetupConfig {
    pub board: BoardConfig,
}
