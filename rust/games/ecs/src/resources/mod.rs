//! 全局资源 —— 阶段 / 事件日志 / RNG / 女巫药水 / 开局配置。
//!
//! 事件日志是唯一事实源（seq = `events` 下标，单调连续）；
//! 它是普通 Resource，与 ECS Message 严格区分（spec §4）。

mod game;

pub use game::{
    DuelOffered, EventLog, GamePhase, GameRng, LastProtected, NightRecord, Potions, SetupConfig,
    Votes, WolfChat, WolfCursor,
};
