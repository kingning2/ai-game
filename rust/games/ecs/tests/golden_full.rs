//! 黄金模糊测试：随机动作序列下 ECS 与旧引擎全程逐字节对拍。
//!
//! 每一步：以引擎 `pending()` 为准选 actor，构造候选动作依次尝试：
//! - 引擎接受 ⇒ ECS 必须接受且增量事件、阶段、全体玩家状态完全一致；
//! - 引擎拒绝 ⇒ ECS 必须同样拒绝（引擎状态不变，继续下一候选）。
//!
//! 覆盖发牌 / 夜聊 / 守卫 / 女巫（含奶穿）/ 预言家 / 发言 / 决斗 / 投票 /
//! 白痴翻牌 / 猎人级联 / 屠边胜负全路径。
//!
//! 注意：`engine`（facade）与直接 World 共用同一套 ECS 实现，本测试守护的是
//! 「facade players() 缓存 ≡ 原始组件」与「facade apply ≡ 直接 apply_action」；
//! 与旧引擎的历史 parity 已在旧引擎存在时的迁移提交中验证。
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use bevy_ecs::prelude::*;
use game_domain::{Action, BoardConfig, NightStage, Phase, Rng};
use game_ecs::components::{DuelUsed, IdiotRevealed, PlayerMark, RoleOf, Seat, SeerChecks};
use game_ecs::resources::GamePhase;
use game_ecs::{apply_action, setup_world};
use game_events::GameEvent;
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

fn full_board() -> BoardConfig {
    // 8 人自定义板：含白痴与骑士（预设三板全为 idiot:0, knight:0，
    // 不补这块则翻牌/决斗路径在对拍中零覆盖）
    BoardConfig {
        werewolves: 2,
        villagers: 2,
        seer: 1,
        witch: 1,
        guard: 0,
        hunter: 0,
        idiot: 1,
        knight: 1,
    }
}

#[test]
fn full_parity_fuzz() {
    let boards: Vec<BoardConfig> = vec![
        BoardConfig::newbie_six(),
        BoardConfig::nine_standard(),
        BoardConfig::all_stars(),
        full_board(),
    ];
    for (bi, board) in boards.into_iter().enumerate() {
        for seed in 0..12u64 {
            drive(board.clone(), (bi as u64) * 1000 + seed);
        }
    }
}

/// 白痴投票权语义：翻牌免死后失去投票权是**永久**的——
/// 后续投票结算不得恢复（回归用例，对应 day.rs 恢复循环的排除条件）
#[test]
fn idiot_loses_vote_permanently() {
    let board = full_board();
    for seed in 0..20u64 {
        let mut engine = GameEngine::new(board.clone(), seed).unwrap();
        let idiot = engine
            .players()
            .iter()
            .find(|p| p.role == game_domain::Role::Idiot)
            .map(|p| p.id)
            .unwrap();

        let day1_exiled = drive_until_vote_target(&mut engine, idiot, 1);
        if !day1_exiled {
            continue; // 该 seed 下白痴未在第 1 天被放逐，换种子
        }
        assert!(engine.players()[idiot as usize].revealed_idiot,
            "seed={seed} 白痴应在第 1 天翻牌"
        );

        // 第 2 天投出任意一名玩家后，白痴仍无投票权（若提前终局则换种子）
        let day2_resolved = drive_until_next_vote_resolved(&mut engine, 2);
        if !day2_resolved {
            continue;
        }
        assert!(!engine.players()[idiot as usize].can_vote,
            "seed={seed} 白痴翻牌后投票权必须永久失去"
        );
        return;
    }
    panic!("20 个种子内没有一个在第 1 天放逐白痴（驱动器可能有 bug）");
}

/// 驱动到「第 `day` 天投票且目标为 `target`」的结算：全员投 target。
/// 返回该次投票是否真的放逐了 target（false = 还没到或结算了别的）。
/// 驱动到「第 `day` 天投票且全员投 `target`」的结算
fn drive_until_vote_target(engine: &mut GameEngine, target: u8, day: u32) -> bool {
    drive_scripted(engine, |phase_day| phase_day == day, target)
}

/// 驱动到第 `day` 天的下一次投票结算
fn drive_until_next_vote_resolved(engine: &mut GameEngine, day: u32) -> bool {
    drive_scripted(engine, |phase_day| phase_day == day, u8::MAX)
}

/// 通用脚本驱动：每步按当前阶段喂最小动作，直到第 `stop` 天的投票结算。
/// `vote_target`：投票窗口的目标（u8::MAX = 各自投自己 → 平票）。
fn drive_scripted(engine: &mut GameEngine, stop: impl Fn(u32) -> bool, vote_target: u8) -> bool {
    for _ in 0..500 {
        let Some((actor, _)) = engine.pending() else {
            return false; // 已终局
        };
        let action = match engine.phase().clone() {
            Phase::Night { stage, day } => night_action(engine, actor, stage, day),
            Phase::DaySpeech { .. } => Action::Speech {
                text: "t".into(),
            },
            Phase::KnightDuel { .. } => Action::KnightDuel { target: None },
            Phase::DayVote { day } => {
                let resolved = drive_day_votes(engine, day, vote_target);
                if stop(day) && resolved {
                    return true;
                }
                continue;
            }
            Phase::HunterShot { .. } => Action::HunterShot { target: None },
            Phase::Settled { .. } => return false,
        };
        if engine.apply(actor, action.clone()).is_err() {
            // 脚本动作可能非法（如女巫解药已耗尽）→ 退化为女巫空过；其它阶段直接失败
            let is_witch_pass = matches!(action, Action::WitchAct { .. });
            assert!(is_witch_pass, "scripted action must be legal: {action:?}");
            let Some((witch_actor, _)) = engine.pending() else {
                return false;
            };
            engine
                .apply(
                    witch_actor,
                    Action::WitchAct {
                        save: false,
                        poison: None,
                    },
                )
                .expect("witch pass must be legal");
        }
    }
    false
}

/// 投票窗口：所有有投票权的活人依次投 `target`，直到结算
fn drive_day_votes(engine: &mut GameEngine, day: u32, target: u8) -> bool {
    loop {
        let Some((actor, _)) = engine.pending() else {
            return false;
        };
        if !matches!(engine.phase(), Phase::DayVote { .. }) {
            return engine
                .events()
                .iter()
                .any(|e| matches!(e, GameEvent::VoteResolved { day: d, .. } if *d == day));
        }
        let vote_target = if target == u8::MAX { actor } else { target };
        engine
            .apply(actor, Action::Vote { target: Some(vote_target) })
            .expect("vote must be legal");
    }
}

/// 夜晚最小合法动作（守卫守自己、狼刀预言家优先、女巫毒狼优先的近似脚本）
fn night_action(engine: &GameEngine, actor: u8, stage: NightStage, _day: u32) -> Action {
    let alive: Vec<u8> = engine
        .players()
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let pick = || alive[0];
    match stage {
        NightStage::Guard => Action::GuardProtect { target: Some(actor) },
        NightStage::WerewolfChat => Action::WolfChat { text: None },
        NightStage::Werewolves => Action::WerewolfKill { target: pick() },
        NightStage::Witch => Action::WitchAct {
            save: true,
            poison: None,
        },
        NightStage::Seer => {
            // 预言家不能验自己
            let target = alive.iter().copied().find(|seat| *seat != actor).unwrap();
            Action::SeerCheck { target }
        }
    }
}
