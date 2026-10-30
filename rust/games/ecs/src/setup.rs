//! 开局装配 —— 校验板子、插入资源、运行 Setup 调度。
//!
//! 这是「每局一个 World」的入口：调用方（后续的 game-runtime）为每局
//! 建独立 `World`，调一次 [`setup_world`] 即完成发牌与资源初始化。
//! 一个 World 只应 setup 一次（二次调用会重复发牌）。

use bevy_ecs::prelude::*;
use game_domain::{BoardConfig, BoardError, Rng};

use crate::flow::enter_night;
use crate::resources::{
    DuelOffered, EventLog, GameRng, LastProtected, NightRecord, Potions, SetupConfig, Votes,
    WolfChat,
};
use crate::schedules::{Setup, setup_schedule};

/// 开局：校验板子 → 插入资源 → 跑 Setup 调度（发牌 + 开局事件）
pub fn setup_world(world: &mut World, board: BoardConfig, seed: u64) -> Result<(), BoardError> {
    board.validate()?;
    let witch_present = board.witch > 0;
    world.insert_resource(SetupConfig { board });
    world.insert_resource(GameRng(Rng::from_seed(seed)));
    world.insert_resource(Potions {
        save: witch_present,
        poison: witch_present,
    });
    world.insert_resource(EventLog::default());
    world.insert_resource(NightRecord::default());
    world.insert_resource(LastProtected(None));
    world.insert_resource(Votes::default());
    world.insert_resource(DuelOffered(false));
    world.insert_resource(WolfChat(None));
    world.init_resource::<Schedules>();
    world.resource_mut::<Schedules>().insert(setup_schedule());
    world.run_schedule(Setup);
    // 与引擎 `GameEngine::new` 一致：发牌事件后立即开启第一夜
    enter_night(world, 1);
    Ok(())
}
