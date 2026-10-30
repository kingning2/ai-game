//! 黄金模糊测试：随机动作序列下 ECS 与旧引擎全程逐字节对拍。
//!
//! 每一步：以引擎 `pending()` 为准选 actor，构造候选动作依次尝试：
//! - 引擎接受 ⇒ ECS 必须接受且增量事件、阶段、全体玩家状态完全一致；
//! - 引擎拒绝 ⇒ ECS 必须同样拒绝（引擎状态不变，继续下一候选）。
//!
//! 覆盖发牌 / 夜聊 / 守卫 / 女巫（含奶穿）/ 预言家 / 发言 / 决斗 / 投票 /
//! 白痴翻牌 / 猎人级联 / 屠边胜负全路径。
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use bevy_ecs::prelude::*;
use game_domain::{
    Action, BoardConfig, NightStage, Phase, Rng,
};
use game_ecs::components::{DuelUsed, IdiotRevealed, PlayerMark, RoleOf, Seat, SeerChecks};
use game_ecs::resources::GamePhase;
use game_ecs::{apply_action, setup_world};
use werewolf::GameEngine;

type Row = (
    u8,
    game_domain::Role,
    bool,
    bool,
    bool,
    bool,
    Vec<game_domain::CheckRecord>,
);

fn ecs_snapshot(world: &mut World) -> (Phase, Vec<Row>) {
    let phase = world.resource::<GamePhase>().0.clone();
    let mut q = world.query::<(
        &PlayerMark,
        &RoleOf,
        &Seat,
        &IdiotRevealed,
        &DuelUsed,
        &SeerChecks,
    )>();
    let mut rows: Vec<Row> = q
        .iter(world)
        .map(|(m, r, s, i, d, c)| (m.0, r.0, s.alive, s.can_vote, i.0, d.0, c.0.clone()))
        .collect();
    rows.sort_by_key(|r| r.0);
    (phase, rows)
}

fn engine_snapshot(engine: &GameEngine) -> (Phase, Vec<Row>) {
    (
        engine.phase().clone(),
        engine
            .players()
            .iter()
            .map(|p| {
                (
                    p.id,
                    p.role,
                    p.alive,
                    p.can_vote,
                    p.revealed_idiot,
                    p.duel_used,
                    p.checks.clone(),
                )
            })
            .collect(),
    )
}

/// 候选动作（含少量非法候选，验证两侧同样拒绝）
fn candidates(engine: &GameEngine, rng: &mut Rng) -> Vec<Action> {
    let alive_seats: Vec<u8> = engine
        .players()
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let pick = |rng: &mut Rng| alive_seats[(rng.next_u64() % alive_seats.len() as u64) as usize];
    match engine.phase() {
        Phase::Night { stage, .. } => match stage {
            NightStage::Guard => vec![
                Action::GuardProtect { target: Some(pick(rng)) },
                Action::GuardProtect { target: None },
            ],
            NightStage::WerewolfChat => vec![
                Action::WolfChat {
                    text: Some(format!("w{}", rng.next_u64() % 4)),
                },
                Action::WolfChat { text: None },
            ],
            NightStage::Werewolves => vec![Action::WerewolfKill { target: pick(rng) }],
            NightStage::Witch => vec![
                Action::WitchAct { save: true, poison: None },
                Action::WitchAct { save: false, poison: None },
                Action::WitchAct { save: false, poison: Some(pick(rng)) },
                Action::WitchAct { save: true, poison: Some(pick(rng)) },
            ],
            NightStage::Seer => vec![Action::SeerCheck { target: pick(rng) }],
        },
        Phase::DaySpeech { .. } => vec![
            Action::Speech {
                text: format!("s{}", rng.next_u64() % 4),
            },
            Action::Speech { text: "  ".into() },
        ],
        Phase::KnightDuel { .. } => vec![
            Action::KnightDuel { target: Some(pick(rng)) },
            Action::KnightDuel { target: None },
        ],
        Phase::DayVote { .. } => vec![
            Action::Vote { target: Some(pick(rng)) },
            Action::Vote { target: None },
        ],
        Phase::HunterShot { .. } => vec![
            Action::HunterShot { target: Some(pick(rng)) },
            Action::HunterShot { target: None },
        ],
        Phase::Settled { .. } => vec![],
    }
}

fn drive(board: BoardConfig, seed: u64) {
    let mut engine = GameEngine::new(board.clone(), seed).unwrap();
    let mut world = World::new();
    setup_world(&mut world, board, seed).unwrap();
    let mut rng = Rng::from_seed(seed ^ 0xF00D);
    let mut steps = 0usize;

    while !engine.settled() && steps < 500 {
        steps += 1;
        let (actor, _) = engine.pending().expect("unsettled game must have a pending actor");
        for action in candidates(&engine, &mut rng) {
            let engine_result = engine.apply(actor, action.clone());
            let ecs_result = apply_action(&mut world, actor, action);
            match engine_result {
                Ok(expected_events) => {
                    let actual = ecs_result.expect("ECS must accept what the engine accepts");
                    assert_eq!(actual, expected_events, "增量事件不一致 seed={seed} step={steps}");
                    assert_eq!(
                        ecs_snapshot(&mut world),
                        engine_snapshot(&engine),
                        "状态不一致 seed={seed} step={steps}"
                    );
                    break;
                }
                Err(expected_err) => {
                    assert_eq!(
                        ecs_result.expect_err("ECS must reject what the engine rejects"),
                        expected_err,
                        "错误不一致 seed={seed} step={steps}"
                    );
                }
            }
        }
    }

    assert!(
        engine.settled(),
        "500 步内未分胜负 seed={seed}（疑似卡死）"
    );
    assert_eq!(
        ecs_snapshot(&mut world),
        engine_snapshot(&engine),
        "终局状态不一致 seed={seed}"
    );
}

#[test]
fn full_parity_fuzz() {
    let boards: Vec<BoardConfig> = vec![
        BoardConfig::newbie_six(),
        BoardConfig::nine_standard(),
        BoardConfig::all_stars(),
    ];
    for (bi, board) in boards.into_iter().enumerate() {
        for seed in 0..12u64 {
            drive(board.clone(), (bi as u64) * 1000 + seed);
        }
    }
}
