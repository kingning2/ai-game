//! Schedule 组织 —— 按游戏阶段划分的调度（spec §5）。
//!
//! 迁移期逐步填充：Setup（发牌）已就位，Night/Day/Voting/Resolution/CheckWin/Cleanup
//! 随系统迁移逐段接入；系统执行顺序一律显式声明，禁止依赖隐式歧义排序。

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;

use crate::systems::player_spawn_system;

/// Setup 阶段标签：开局校验后发牌
#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Setup;

/// 构建 Setup 调度：PlayerSpawnSystem
pub fn setup_schedule() -> Schedule {
    let mut schedule = Schedule::new(Setup);
    schedule.add_systems(player_spawn_system);
    schedule
}
