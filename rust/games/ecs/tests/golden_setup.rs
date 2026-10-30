//! 黄金测试：同 seed 下 ECS 发牌与旧引擎逐座位一致（迁移期行为守护）。
//!
//! 对每个板子 × 每个种子断言：
//! 1. 逐座位 (id, role, alive, can_vote) 与 `GameEngine::new` 完全一致；
//! 2. 开局事件日志（GameStarted + NightBegan）与引擎逐字节一致。

use bevy_ecs::prelude::*;
use game_domain::BoardConfig;
use game_ecs::components::{PlayerMark, RoleOf, Seat};
use game_ecs::resources::EventLog;
use game_ecs::setup::setup_world;
use werewolf::GameEngine;

const BOARDS: &[fn() -> BoardConfig] = &[
    BoardConfig::newbie_six,
    BoardConfig::nine_standard,
    BoardConfig::all_stars,
];

const SEEDS: &[u64] = &[0, 1, 42, 999];

#[test]
fn same_seed_same_deal() {
    for board_fn in BOARDS {
        let board = board_fn();
        for &seed in SEEDS {
            let engine = GameEngine::new(board.clone(), seed).unwrap();
            let mut world = World::new();
            setup_world(&mut world, board.clone(), seed).unwrap();

            let mut query = world.query::<(&PlayerMark, &RoleOf, &Seat)>();
            let mut ecs_deal: Vec<_> = query
                .iter(&world)
                .map(|(mark, role, seat)| (mark.0, role.0, seat.alive, seat.can_vote))
                .collect();
            ecs_deal.sort_by_key(|&(id, ..)| id);

            let engine_deal: Vec<_> = engine
                .players()
                .iter()
                .map(|p| (p.id, p.role, p.alive, p.can_vote))
                .collect();

            assert_eq!(ecs_deal, engine_deal, "发牌不一致: seed={seed}");

            let log = world.resource::<EventLog>();
            // 全量比对开局日志，不只第一条
            assert_eq!(log.events, engine.events(), "开局日志不一致: seed={seed}");
        }
    }
}
