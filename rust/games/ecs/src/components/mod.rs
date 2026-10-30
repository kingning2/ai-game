//! 玩家组件 —— `PlayerState` 的 ECS 拆分镜像。
//!
//! 每字段一组件，禁止把整个 PlayerState 塞成一个大组件（spec §4）；
//! 字段语义与 `werewolf::player::PlayerState` 逐字对齐，迁移期两侧黄金测试对齐。

mod player;

pub use player::{DuelUsed, IdiotRevealed, PlayerBundle, PlayerMark, RoleOf, Seat, SeerChecks};
