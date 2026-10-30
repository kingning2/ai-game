//! 资源定义：游戏阶段 / 事件日志 / RNG / 女巫药水 / 开局配置 / 昼夜累计状态。
//!
//! 命名与语义同 `werewolf::engine` 对应私有字段（迁移期黄金测试守护一致性）。

use bevy_ecs::prelude::*;
use game_domain::{BoardConfig, Phase, PlayerId, Rng};
use game_events::GameEvent;

/// 游戏阶段（与引擎 `phase` 字段等价）
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

/// 开局配置（Setup 阶段消费一次；`board.total()` 全程可查座位数）
#[derive(Resource, Debug, Clone)]
pub struct SetupConfig {
    pub board: BoardConfig,
}

/// 单晚夜间动作累计（天亮结算后清空）
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NightRecord {
    /// 守卫守护目标
    pub protected: Option<PlayerId>,
    /// 狼队刀口
    pub killed: Option<PlayerId>,
    /// 女巫是否用了解药
    pub saved: bool,
    /// 女巫毒杀目标
    pub poisoned: Option<PlayerId>,
}

/// 昨晚守护目标（守卫不能连守同一人）
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LastProtected(pub Option<PlayerId>);

/// 本轮白天票箱（逐个收集，结算后清空）
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct Votes(pub Vec<Option<PlayerId>>);

/// 骑士决斗窗口本轮白天是否已出现（一次；入昼时重置）
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DuelOffered(pub bool);

/// 狼队夜聊轮转游标（仅 [`game_domain::NightStage::WerewolfChat`] 段非空；
/// 从第一个活狼起按座位序绕一圈，回到起点即结束）
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WolfChat(pub Option<WolfCursor>);

/// 狼队夜聊游标状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WolfCursor {
    pub cursor: PlayerId,
    pub start: PlayerId,
}
