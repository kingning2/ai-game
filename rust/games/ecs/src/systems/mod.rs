//! 游戏 System —— 按阶段拆分的规则执行单元（spec §5）。

mod spawn;

pub use spawn::player_spawn_system;
