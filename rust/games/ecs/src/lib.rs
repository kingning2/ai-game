//! 狼人杀 ECS 运行时 —— bevy_ecs 数据组织与 System 调度（迁移阶段 3 起步）。
//!
//! 边界（spec §2）：
//! - 游戏领域规则在 `game-domain`（零 bevy 依赖）；业务事实在 `game-events`。
//! - 每个玩家一个 Entity，业务身份用 [`game_domain::phase::PlayerId`]，Bevy Entity 仅内部关联。
//! - 事件日志是普通 [`Resource`](bevy_ecs::resource::Resource)，不是 ECS Message。
//! - 对外 World 不暴露：server / agent 只经 werewolf 门面与 game-runtime 交互。
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod apply;
pub mod components;
pub(crate) mod day;
pub(crate) mod flow;
pub(crate) mod night;
pub mod resources;
pub mod schedules;
pub mod setup;
pub mod systems;

pub use apply::apply_action;
pub use setup::setup_world;
