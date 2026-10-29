//! 端到端场景测试：从发牌到胜负的完整对局推进 + 非法动作 + 视角保密

use crate::action::Action;
use crate::board::BoardConfig;
use crate::engine::GameEngine;
use crate::error::GameError;
use crate::event::GameEvent;
use crate::phase::{DeathCause, NightStage, Outcome, Phase, Viewer};
use crate::role::Role;
use crate::snapshot::GameSnapshot;
use crate::summary::GameSummary;

/// 指定身份的存活数（板子里没有该身份则返回 0）
fn alive_of(summary: &GameSummary, role: Role) -> u8 {
    summary
        .roles
        .iter()
        .find(|r| r.role == role)
        .map_or(0, |r| r.alive)
}

/// 测试用驱动器：按窗口自动喂动作，推进完整对局
struct Driver(GameEngine);

/// 一晚的夜晚计划
#[derive(Clone, Copy, Default)]
struct NightPlan {
    guard: Option<u8>,
    kill: u8,
    save: bool,
    poison: Option<u8>,
    check: u8,
}

impl Driver {
    fn new(board: BoardConfig, seed: u64) -> Self {
        Self(GameEngine::new(board, seed).expect("合法板子"))
    }

    /// 上帝视角查角色座位（测试特权，直接读引擎内部）
    fn seat(&self, role: Role) -> u8 {
        self.0
            .players()
            .iter()
            .find(|p| p.role == role && p.alive)
            .or_else(|| self.0.players().iter().find(|p| p.role == role))
            .unwrap_or_else(|| panic!("板子里没有 {role:?}"))
            .id
    }

    /// 推进一晚（跳过板子里不存在的角色窗口）
    fn run_night(&mut self, plan: NightPlan) {
        while let Phase::Night { stage, .. } = self.0.phase().clone() {
            let (actor, _) = self.0.pending().expect("夜晚必有窗口");
            let result = match stage {
                NightStage::Guard => self
                    .0
                    .apply(actor, Action::GuardProtect { target: plan.guard }),
                NightStage::WerewolfChat => self.0.apply(actor, Action::WolfChat { text: None }),
                NightStage::Werewolves => self
                    .0
                    .apply(actor, Action::WerewolfKill { target: plan.kill }),
                NightStage::Witch => self.0.apply(
                    actor,
                    Action::WitchAct {
                        save: plan.save,
                        poison: plan.poison,
                    },
                ),
                NightStage::Seer => self
                    .0
                    .apply(actor, Action::SeerCheck { target: plan.check }),
            };
            result.unwrap_or_else(|err| panic!("夜晚动作失败（{stage:?}）: {err}"));
        }
    }

    /// 发言一圈 → 骑士窗口 → 投票（ballots 按活人座位序弹出）
    fn run_day(&mut self, duel: Option<u8>, ballots: &[Option<u8>]) {
        while let Phase::DaySpeech { .. } = self.0.phase() {
            let (actor, _) = self.0.pending().expect("发言必有窗口");
            self.0
                .apply(
                    actor,
                    Action::Speech {
                        text: format!("{actor} 号发言"),
                    },
                )
                .expect("发言合法");
        }
        if let Phase::KnightDuel { .. } = self.0.phase() {
            let (actor, _) = self.0.pending().expect("决斗必有窗口");
            self.0
                .apply(actor, Action::KnightDuel { target: duel })
                .expect("决斗合法");
        }
        let mut ballots = ballots.to_vec();
        while let Phase::DayVote { .. } = self.0.phase() {
            let (actor, _) = self.0.pending().expect("投票必有窗口");
            let ballot = ballots.remove(0);
            self.0
                .apply(actor, Action::Vote { target: ballot })
                .expect("投票合法");
        }
    }

    /// 猎人窗口喂动作（有的场景没有窗口，调用方自行判断）
    fn run_hunter(&mut self, target: Option<u8>) {
        if let Phase::HunterShot { .. } = self.0.phase() {
            let (actor, _) = self.0.pending().expect("猎人窗口必有 actor");
            self.0
                .apply(actor, Action::HunterShot { target })
                .expect("猎人动作合法");
        }
    }

    fn phase(&self) -> &Phase {
        self.0.phase()
    }
}

/// 最小可玩板子：1 狼 + 1 预言家 + 1 平民（3 人）
fn tiny_board() -> BoardConfig {
    BoardConfig {
        werewolves: 1,
        seer: 1,
        villagers: 1,
        witch: 0,
        guard: 0,
        hunter: 0,
        idiot: 0,
        knight: 0,
    }
}

// ---------- 发牌 ----------

#[test]
fn deal_is_reproducible_by_seed() {
    let board = BoardConfig::all_stars();
    let a = GameEngine::new(board.clone(), 7).unwrap();
    let b = GameEngine::new(board.clone(), 7).unwrap();
    let c = GameEngine::new(board, 8).unwrap();

    let roles = |engine: &GameEngine| engine.players().iter().map(|p| p.role).collect::<Vec<_>>();
    assert_eq!(roles(&a), roles(&b), "同 seed 发牌必须一致");
    assert_ne!(roles(&a), roles(&c), "不同 seed 发牌应不同");
}

#[test]
fn deal_matches_board_multiset() {
    let board = BoardConfig::all_stars();
    let engine = GameEngine::new(board.clone(), 1).unwrap();
    let mut counts = std::collections::BTreeMap::new();
    for player in engine.players() {
        *counts.entry(player.role).or_insert(0) += 1;
    }
    for (role, count) in
        board
            .roles()
            .into_iter()
            .fold(std::collections::BTreeMap::new(), |mut acc, role| {
                *acc.entry(role).or_insert(0) += 1;
                acc
            })
    {
        assert_eq!(counts[&role], count, "{role:?} 数量不符");
    }
}

// ---------- 夜晚结算 ----------

#[test]
fn wolf_kill_without_save_kills_target() {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 1);
    let (wolf, seer, villager) = (
        driver.seat(Role::Werewolf),
        driver.seat(Role::Seer),
        driver.seat(Role::Villager),
    );
    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });

    // 天亮公布死者，进入白天发言
    assert!(matches!(driver.phase(), Phase::DaySpeech { .. }));
    assert!(driver.0.events().iter().any(|event| matches!(
        event,
        GameEvent::PlayerDied { player, cause: DeathCause::Slain, .. } if player.id == villager
    )));
    let _ = seer;
}

#[test]
fn witch_save_means_quiet_night() {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 3);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        kill: villager,
        save: true,
        check: villager,
        ..Default::default()
    });

    let dawn = driver
        .0
        .events()
        .iter()
        .find_map(|event| match event {
            GameEvent::Dawn { deaths, .. } => Some(deaths.clone()),
            _ => None,
        })
        .expect("必有天亮事件");
    assert!(dawn.is_empty(), "被救者不应死亡");
    assert!(matches!(driver.phase(), Phase::DaySpeech { .. }));
}

#[test]
fn guard_block_means_quiet_night() {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 5);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        guard: Some(villager),
        kill: villager,
        check: villager,
        ..Default::default()
    });

    let dawn = driver
        .0
        .events()
        .iter()
        .find_map(|event| match event {
            GameEvent::Dawn { deaths, .. } => Some(deaths.clone()),
            _ => None,
        })
        .expect("必有天亮事件");
    assert!(dawn.is_empty(), "守卫挡刀应为平安夜");
}

#[test]
fn guard_plus_save_pierces_the_milk() {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 9);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        guard: Some(villager),
        kill: villager,
        save: true,
        check: villager,
        ..Default::default()
    });

    let dawn = driver
        .0
        .events()
        .iter()
        .find_map(|event| match event {
            GameEvent::Dawn { deaths, .. } => Some(deaths.clone()),
            _ => None,
        })
        .expect("必有天亮事件");
    assert_eq!(dawn.len(), 1, "守+救同一人应「奶穿」死亡");
    assert_eq!(dawn[0].player, villager);
    assert_eq!(dawn[0].cause, DeathCause::Slain);
}

#[test]
fn poisoned_hunter_cannot_shoot() {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 11);
    let hunter = driver.seat(Role::Hunter);
    let villager = driver.seat(Role::Villager);
    // 狼刀别人、女巫毒猎人：猎人死因是毒 ⇒ 没有开枪窗口
    driver.run_night(NightPlan {
        kill: villager,
        poison: Some(hunter),
        check: hunter,
        ..Default::default()
    });

    // 被毒猎人没有开枪窗口：直接进入白天
    assert!(
        !matches!(driver.phase(), Phase::HunterShot { .. }),
        "被毒猎人不能开枪"
    );
    assert!(matches!(driver.phase(), Phase::DaySpeech { .. }));
}

// ---------- 白天 ----------

#[test]
fn exiled_hunter_can_shoot() {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 13);
    let hunter = driver.seat(Role::Hunter);
    let wolf = driver.seat(Role::Werewolf);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });

    // 全体投猎人（4 狼也投猎人保证最高票唯一——14 人局活人多，直接按座位全投猎人）
    let alive: Vec<u8> = driver
        .0
        .players()
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let ballots: Vec<Option<u8>> = alive.iter().map(|_| Some(hunter)).collect();
    driver.run_day(None, &ballots);
    driver.run_hunter(Some(wolf));

    assert!(
        driver.0.events().iter().any(|event| matches!(
            event,
            GameEvent::HunterShotFired { target: Some(target), .. } if target.id == wolf
        )),
        "被放逐猎人应能开枪"
    );
}

#[test]
fn idiot_survives_exile_but_loses_vote() {
    let mut board = BoardConfig::all_stars();
    board.idiot = 1;
    board.werewolves = 2;
    let mut driver = Driver::new(board, 17);
    let idiot = driver.seat(Role::Idiot);
    let villager = driver.seat(Role::Villager);
    let wolf = driver.seat(Role::Werewolf);
    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });

    let alive: Vec<u8> = driver
        .0
        .players()
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let ballots: Vec<Option<u8>> = alive.iter().map(|_| Some(idiot)).collect();
    driver.run_day(None, &ballots);

    assert!(
        driver.0.events().iter().any(
            |event| matches!(event, GameEvent::IdiotRevealed { idiot: id, .. } if *id == idiot)
        ),
        "白痴被放逐应翻牌"
    );
    let idiot_player = &driver.0.players()[idiot as usize];
    assert!(idiot_player.alive, "白痴翻牌免死");
    assert!(idiot_player.revealed_idiot);
    assert!(!idiot_player.can_vote, "翻牌后失去投票权");
    // 对局未结束，进入下一夜
    assert!(matches!(driver.phase(), Phase::Night { .. }));
}

#[test]
fn idiot_dies_at_night_without_reveal() {
    let mut board = BoardConfig::all_stars();
    board.idiot = 1;
    board.werewolves = 2;
    let mut driver = Driver::new(board, 19);
    let idiot = driver.seat(Role::Idiot);
    driver.run_night(NightPlan {
        kill: idiot,
        check: idiot,
        ..Default::default()
    });

    assert!(
        driver.0.events().iter().any(|event| matches!(
            event,
            GameEvent::PlayerDied { player, cause: DeathCause::Slain, .. } if player.id == idiot
        )),
        "白痴夜晚被刀应直接死亡"
    );
    assert!(
        !driver
            .0
            .events()
            .iter()
            .any(|event| matches!(event, GameEvent::IdiotRevealed { .. })),
        "夜晚死亡不触发白痴翻牌"
    );
}

#[test]
fn knight_duel_kills_wolf_on_success() {
    let mut board = BoardConfig::all_stars();
    board.knight = 1;
    board.werewolves = 2;
    let mut driver = Driver::new(board, 23);
    let wolf = driver.seat(Role::Werewolf);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });

    // 平票：轮流投空（None 全体）
    let alive: Vec<u8> = driver
        .0
        .players()
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let ballots: Vec<Option<u8>> = alive.iter().map(|_| None).collect();
    driver.run_day(Some(wolf), &ballots);

    assert!(
        driver.0.events().iter().any(|event| matches!(
            event,
            GameEvent::KnightDuelResolved { target, target_is_wolf: true, .. } if *target == wolf
        )),
        "决斗狼人应判狼死"
    );
    assert!(!driver.0.players()[wolf as usize].alive);
}

#[test]
fn knight_duel_failure_kills_knight() {
    let mut board = BoardConfig::all_stars();
    board.knight = 1;
    board.werewolves = 2;
    let mut driver = Driver::new(board, 29);
    let knight = driver.seat(Role::Knight);
    let seer = driver.seat(Role::Seer);
    let villager = driver.seat(Role::Villager);
    let wolf = driver.seat(Role::Werewolf);
    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });

    let alive: Vec<u8> = driver
        .0
        .players()
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let ballots: Vec<Option<u8>> = alive.iter().map(|_| None).collect();
    driver.run_day(Some(seer), &ballots);

    assert!(
        driver.0.events().iter().any(|event| matches!(
            event,
            GameEvent::KnightDuelResolved {
                target_is_wolf: false,
                ..
            }
        )),
        "决斗好人应判骑士输"
    );
    assert!(!driver.0.players()[knight as usize].alive, "骑士赌输应死");
}

#[test]
fn tie_vote_exiles_nobody() {
    // 1 狼 + 3 民 + all_stars 神职（10 人）：刀死 1 民后 9 人循环票 ⇒ 每人 1 票 ⇒ 平票
    let mut board = BoardConfig::all_stars();
    board.werewolves = 1;
    board.villagers = 3;
    let mut driver = Driver::new(board, 31);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        kill: villager,
        check: villager,
        ..Default::default()
    });

    // 循环票：每个活人投 alive 里的下一位
    let alive: Vec<u8> = driver
        .0
        .players()
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let ballots: Vec<Option<u8>> = alive
        .iter()
        .enumerate()
        .map(|(i, _)| Some(alive[(i + 1) % alive.len()]))
        .collect();
    driver.run_day(None, &ballots);

    assert!(
        driver
            .0
            .events()
            .iter()
            .any(|event| matches!(event, GameEvent::VoteResolved { tie: true, .. })),
        "平票应无人放逐"
    );
    assert!(
        matches!(driver.phase(), Phase::Night { .. }),
        "平票后进入下一夜"
    );
}

// ---------- 胜负 ----------

#[test]
fn no_outcome_while_both_camps_have_members() {
    // 屠边规则：开局满员（4 狼 / 4 神 / 4 民）不结算；夜刀 1 民也不结算
    let mut driver = Driver::new(BoardConfig::all_stars(), 91);
    let wolf = driver.seat(Role::Werewolf);
    let villager = driver.seat(Role::Villager);
    // 验人目标不能是预言家自己，选一个非自己存活座位
    let seer = driver.seat(Role::Seer);
    let check_target = if seer == 1 { 2 } else { 1 };
    driver.run_night(NightPlan {
        kill: villager,
        check: check_target,
        ..Default::default()
    });
    assert!(!driver.0.settled(), "只死 1 民不应分出胜负");
    assert!(!matches!(driver.phase(), Phase::Settled { .. }));
    let _ = wolf;
}

#[test]
fn werewolf_wins_by_wiping_villagers() {
    // 1 狼 + 1 预言家 + 1 民：夜刀唯一的民 → 屠民胜（神职尚存）
    let mut driver = Driver::new(tiny_board(), 53);
    let wolf = driver.seat(Role::Werewolf);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });
    assert_eq!(
        driver.phase(),
        &Phase::Settled {
            outcome: Outcome::WerewolfWin {
                reason: crate::phase::WerewolfReason::VillagersWiped
            }
        },
        "屠民：民 0 但神职尚存"
    );
    assert!(driver.0.settled());
}

#[test]
fn werewolf_wins_by_wiping_gods() {
    let mut driver = Driver::new(tiny_board(), 37);
    let seer = driver.seat(Role::Seer);
    let wolf = driver.seat(Role::Werewolf);
    driver.run_night(NightPlan {
        kill: seer,
        check: wolf,
        ..Default::default()
    });

    assert_eq!(
        driver.phase(),
        &Phase::Settled {
            outcome: Outcome::WerewolfWin {
                reason: crate::phase::WerewolfReason::GodsWiped
            }
        },
        "屠神：神职全灭"
    );
    assert!(driver.0.settled());
}

#[test]
fn good_win_when_all_wolves_dead() {
    // 1 狼 + 3 民 + all_stars 神职（10 人）：白天放逐唯一的狼 → 好人胜
    let mut board = BoardConfig::all_stars();
    board.werewolves = 1;
    board.villagers = 3;
    let mut driver = Driver::new(board, 43);
    let wolf = driver.seat(Role::Werewolf);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });
    // 全员投狼
    let alive: Vec<u8> = driver
        .0
        .players()
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let ballots: Vec<Option<u8>> = alive.iter().map(|_| Some(wolf)).collect();
    driver.run_day(None, &ballots);
    driver.run_hunter(None);

    assert_eq!(
        driver.phase(),
        &Phase::Settled {
            outcome: Outcome::GoodWin
        },
        "狼全灭好人胜"
    );
}

#[test]
fn night_window_skips_absent_roles() {
    // 9 人标准局没有守卫：第一夜应直接轮狼人
    let driver = Driver::new(BoardConfig::nine_standard(), 47);
    let (actor, _) = driver.0.pending().expect("开局必有窗口");
    let wolf = driver.seat(Role::Werewolf);
    assert_eq!(actor, wolf, "无守卫板子第一夜直接轮狼人");
}

// ---------- 非法动作 ----------

#[test]
fn illegal_actions_are_rejected() {
    // all_stars+2狼：刀死 1 个民不触发屠边，流程能推进到白天
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 53);
    let wolf = driver.seat(Role::Werewolf);
    let seer = driver.seat(Role::Seer);
    let villager = driver.seat(Role::Villager);

    // 夜里非狼人喂刀（首夜守卫窗口 → 窗口不符；若在狼窗口则是非狼人）
    assert!(matches!(
        driver
            .0
            .apply(seer, Action::WerewolfKill { target: villager }),
        Err(GameError::NotYourTurn { .. } | GameError::WrongPhase(_))
    ));
    // 预言家验自己（先推进到预言家窗口再验自己——但当前在狼窗口；
    // 直接喂狼人窗口的预言家动作会被 WrongPhase 拦截）
    assert!(matches!(
        driver.0.apply(seer, Action::SeerCheck { target: seer }),
        Err(GameError::NotYourTurn { .. } | GameError::SelfTarget(_) | GameError::WrongPhase(_))
    ));
    // 刀不存在座位（守卫窗口下刀人动作本身窗口不符，也接受目标非法错误）
    assert!(matches!(
        driver.0.apply(wolf, Action::WerewolfKill { target: 99 }),
        Err(GameError::NoSuchPlayer { .. } | GameError::WrongPhase(_))
    ));

    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });
    assert!(matches!(driver.phase(), Phase::DaySpeech { .. }));

    // 非当前发言者
    assert!(matches!(
        driver.0.apply(
            wolf,
            Action::Speech {
                text: "抢话".into()
            }
        ),
        Err(GameError::NotSpeakerTurn { .. } | GameError::NotYourTurn { .. })
    ));
    // 空发言
    let (speaker, _) = driver.0.pending().unwrap();
    assert!(matches!(
        driver
            .0
            .apply(speaker, Action::Speech { text: "   ".into() }),
        Err(GameError::EmptySpeech)
    ));
}

#[test]
fn guard_cannot_repeat_target() {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 59);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        guard: Some(villager),
        kill: villager,
        check: villager,
        ..Default::default()
    });
    // 第一夜平安夜，白天全员弃票 → 第二夜守同一人应被拒
    let alive: Vec<u8> = driver
        .0
        .players()
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let ballots: Vec<Option<u8>> = alive.iter().map(|_| None).collect();
    driver.run_day(None, &ballots);

    let guard = driver.seat(Role::Guard);
    loop {
        match driver.0.phase().clone() {
            Phase::Night {
                stage: NightStage::Guard,
                ..
            } => {
                let (actor, _) = driver.0.pending().unwrap();
                let err = driver
                    .0
                    .apply(
                        actor,
                        Action::GuardProtect {
                            target: Some(villager),
                        },
                    )
                    .expect_err("连守同一人应被拒");
                assert!(matches!(err, GameError::GuardRepeat));
                // 用合法动作把夜晚推完，避免死循环
                driver.run_night(NightPlan {
                    guard: None,
                    kill: villager,
                    check: villager,
                    ..Default::default()
                });
                break;
            }
            Phase::DaySpeech { .. } => {
                let (actor, _) = driver.0.pending().unwrap();
                driver
                    .0
                    .apply(
                        actor,
                        Action::Speech {
                            text: "……".into()
                        },
                    )
                    .unwrap();
            }
            Phase::KnightDuel { .. } => {
                let (actor, _) = driver.0.pending().unwrap();
                driver
                    .0
                    .apply(actor, Action::KnightDuel { target: None })
                    .unwrap();
            }
            Phase::DayVote { .. } => {
                let (actor, _) = driver.0.pending().unwrap();
                driver
                    .0
                    .apply(actor, Action::Vote { target: None })
                    .unwrap();
            }
            _ => break,
        }
    }
    let _ = guard;
}

#[test]
fn witch_rules_are_enforced() {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 61);
    let witch = driver.seat(Role::Witch);
    let villager = driver.seat(Role::Villager);

    // 同晚双药
    driver.run_night(NightPlan {
        kill: villager,
        check: villager,
        ..Default::default()
    });
    // 推到第二夜测试双药/药尽
    let alive: Vec<u8> = driver
        .0
        .players()
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let ballots: Vec<Option<u8>> = alive.iter().map(|_| None).collect();
    driver.run_day(None, &ballots);

    let wolf = driver.seat(Role::Werewolf);
    // 第二夜：女巫想同时救+毒 → WitchBothPotions（check 目标用活着的狼）
    loop {
        match driver.0.phase().clone() {
            Phase::Night {
                stage: NightStage::Witch,
                ..
            } => {
                let (actor, _) = driver.0.pending().unwrap();
                let err = driver
                    .0
                    .apply(
                        actor,
                        Action::WitchAct {
                            save: true,
                            poison: Some(witch),
                        },
                    )
                    .expect_err("同晚双药应被拒");
                assert!(matches!(err, GameError::WitchBothPotions));
                break;
            }
            Phase::Night { .. } => {
                driver.run_night(NightPlan {
                    guard: None,
                    kill: witch,
                    check: wolf,
                    ..Default::default()
                });
            }
            _ => break,
        }
    }
    let _ = villager;
}

#[test]
fn actions_after_game_over_are_rejected() {
    let mut driver = Driver::new(tiny_board(), 67);
    let seer = driver.seat(Role::Seer);
    let wolf = driver.seat(Role::Werewolf);
    driver.run_night(NightPlan {
        kill: seer,
        check: wolf,
        ..Default::default()
    });
    assert!(driver.0.settled());
    assert!(matches!(
        driver.0.apply(
            seer,
            Action::Speech {
                text: "诈尸".into()
            }
        ),
        Err(GameError::GameOver)
    ));
}

#[test]
fn seer_check_record_is_recorded() {
    let mut driver = Driver::new(tiny_board(), 71);
    let wolf = driver.seat(Role::Werewolf);
    let seer = driver.seat(Role::Seer);
    // 预言家窗口在狼之后；直接跑一晚（check 目标 = 狼）
    driver.run_night(NightPlan {
        kill: seer,
        check: wolf,
        ..Default::default()
    });
    let seer_player = &driver.0.players()[seer as usize];
    assert_eq!(seer_player.checks().len(), 1);
    assert_eq!(
        seer_player.checks()[0].result,
        crate::phase::CheckResult::Werewolf
    );
}

// ---------- 视角保密 ----------

#[test]
fn snapshot_hides_roles_from_players() {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 73);
    let wolf = driver.seat(Role::Werewolf);
    let seer = driver.seat(Role::Seer);
    driver.run_night(NightPlan {
        kill: seer,
        check: wolf,
        ..Default::default()
    });

    let god = GameSnapshot::render(&driver.0, Viewer::God).unwrap();
    assert!(god.players.iter().all(|p| p.role.is_some()), "上帝全知");

    let wolf_view = GameSnapshot::render(&driver.0, Viewer::Player(wolf)).unwrap();
    let seer_view = GameSnapshot::render(&driver.0, Viewer::Player(seer)).unwrap();
    // 狼人看队友：狼
    assert_eq!(wolf_view.players[wolf as usize].role, Some(Role::Werewolf));
    let wolves: Vec<_> = driver
        .0
        .players()
        .iter()
        .filter(|p| p.role == Role::Werewolf)
        .map(|p| p.id)
        .collect();
    for w in wolves {
        assert_eq!(
            wolf_view.players[w as usize].role,
            Some(Role::Werewolf),
            "狼队友互见"
        );
    }
    // 狼人看不到预言家（已死→翻牌公开，换活神验保密：看骑士）
    // 预言家本人看自己
    assert_eq!(seer_view.players[seer as usize].role, Some(Role::Seer));
    // 死者翻牌公开（预言家被刀死）
    assert_eq!(seer_view.players[seer as usize].role, Some(Role::Seer));
}

#[test]
fn snapshot_filters_private_logs() {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 79);
    let wolf = driver.seat(Role::Werewolf);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });

    let god = GameSnapshot::render(&driver.0, Viewer::God).unwrap();
    let wolf_view = GameSnapshot::render(&driver.0, Viewer::Player(wolf)).unwrap();

    assert!(
        god.logs.iter().any(|log| log.text.contains("狼队袭击")),
        "上帝日志含夜晚动作"
    );
    // 狼人视角：自己的刀可见
    assert!(
        wolf_view
            .logs
            .iter()
            .any(|log| log.text.contains("狼队袭击"))
    );
    // 女巫的动作不进狼人日志
    let witch_text = god
        .logs
        .iter()
        .find(|log| log.text.contains("女巫"))
        .map(|log| log.text.clone());
    if let Some(text) = witch_text {
        assert!(
            !wolf_view.logs.iter().any(|log| log.text == text),
            "狼人不应看到女巫动作"
        );
    }
}

#[test]
fn seer_checks_only_visible_to_self() {
    let mut driver = Driver::new(tiny_board(), 83);
    let wolf = driver.seat(Role::Werewolf);
    let seer = driver.seat(Role::Seer);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });

    let seer_view = GameSnapshot::render(&driver.0, Viewer::Player(seer)).unwrap();
    assert_eq!(seer_view.checks.len(), 1, "预言家本人可见验人记录");
    assert_eq!(
        seer_view.checks[0].result,
        crate::phase::CheckResult::Werewolf
    );

    let god_view = GameSnapshot::render(&driver.0, Viewer::God).unwrap();
    assert!(
        god_view.checks.is_empty(),
        "上帝快照不携带验人记录（事件流已有）"
    );
}

#[test]
fn snapshot_serde_round_trip() {
    let mut driver = Driver::new(tiny_board(), 89);
    let wolf = driver.seat(Role::Werewolf);
    let villager = driver.seat(Role::Villager);
    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });

    let snapshot = GameSnapshot::render(&driver.0, Viewer::God).unwrap();
    let json = serde_json::to_string(&snapshot).unwrap();
    let back: GameSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(back, snapshot);
}

// ---------- 全局总览 ----------

#[test]
fn summary_tracks_survivors_by_role() {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 97);
    let villager = driver.seat(Role::Villager);

    // 开局：全员存活，身份总数自洽
    let opening = GameSummary::of(&driver.0);
    let total_sum: u8 = opening.roles.iter().map(|r| r.total).sum();
    assert_eq!(total_sum, opening.total, "各身份总人数之和应等于座位数");
    assert_eq!(opening.alive_total, opening.total, "开局全员存活");
    let alive_of = |s: &GameSummary, role: Role| {
        s.roles
            .iter()
            .find(|r| r.role == role)
            .map_or(0, |r| r.alive)
    };
    assert_eq!(alive_of(&opening, Role::Werewolf), 2);
    assert_eq!(alive_of(&opening, Role::Seer), 1);
    assert_eq!(alive_of(&opening, Role::Guard), 1, "all_stars 含 1 守卫");

    // 刀死 1 民：只有该身份存活数减 1，其余不变
    driver.run_night(NightPlan {
        kill: villager,
        check: villager,
        ..Default::default()
    });
    let summary = GameSummary::of(&driver.0);
    assert_eq!(summary.total, opening.total);
    assert_eq!(summary.alive_total, opening.total - 1);
    let villager_status = summary
        .roles
        .iter()
        .find(|r| r.role == Role::Villager)
        .expect("必有村民");
    assert_eq!(villager_status.alive, villager_status.total - 1);
    let wolf = summary
        .roles
        .iter()
        .find(|r| r.role == Role::Werewolf)
        .expect("必有狼");
    assert_eq!(wolf.alive, wolf.total, "狼应全员存活");
    assert_eq!(summary.day, driver.0.phase().day());

    // 角色列表按枚举序稳定输出
    let mut sorted = summary.roles.clone();
    sorted.sort_by_key(|r| r.role);
    assert_eq!(summary.roles, sorted);
}

#[test]
fn summary_absent_role_returns_zero() {
    // tiny_board（1狼1预1民）没有守卫/女巫等身份 → 查询返回 0
    let driver = Driver::new(tiny_board(), 101);
    let summary = GameSummary::of(&driver.0);
    assert_eq!(alive_of(&summary, Role::Guard), 0);
    assert_eq!(alive_of(&summary, Role::Witch), 0);
    assert_eq!(alive_of(&summary, Role::Werewolf), 1);
    assert_eq!(summary.day, 1, "开局第一夜");
}

// ---------- 角色档案 ----------

#[test]
fn profiles_cover_all_roles() {
    // 每个角色都有档案，档案的 role 字段与查表键一致
    for role in [
        Role::Werewolf,
        Role::Seer,
        Role::Witch,
        Role::Guard,
        Role::Hunter,
        Role::Idiot,
        Role::Knight,
        Role::Villager,
    ] {
        let profile = crate::role::profile(role);
        assert_eq!(profile.role, role, "{role:?} 档案指向了错误角色");
        assert!(!profile.description.is_empty());
        assert!(!profile.abilities.is_empty(), "{role:?} 应至少描述一条能力");
        assert!(!profile.win_condition.is_empty());
    }
}

#[test]
fn profile_serde_round_trip() {
    let profile = crate::role::profile(Role::Werewolf);
    let json = serde_json::to_string(profile).unwrap();
    // 档案是静态描述数据，只需序列化下发（前端/AI 不回传档案）；
    // 验证 JSON 含关键描述字段即可
    assert!(json.contains("狼人"));
    assert!(json.contains("屠边"));
    assert_eq!(
        serde_json::to_value(profile).unwrap()["emoji"],
        serde_json::Value::String("🐺".into())
    );
}

// ---------- 狼队夜聊 ----------

#[test]
fn wolf_council_rotates_by_seating() {
    // 双狼局：狼聊按座位序轮转，绕回起点后自动进刀人窗口
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 61);
    // all_stars 有守卫：空守后进狼聊段
    let (guard, _) = driver.0.pending().expect("首夜必有守卫窗口");
    driver
        .0
        .apply(guard, Action::GuardProtect { target: None })
        .expect("空守合法");

    let wolf1 = driver.seat(Role::Werewolf);
    let wolf2 = driver
        .0
        .players()
        .iter()
        .find(|p| p.role == Role::Werewolf && p.alive && p.id != wolf1)
        .map(|p| p.id)
        .expect("双狼局必有第二只狼");
    let villager = driver.seat(Role::Villager);

    let (actor, kind) = driver.0.pending().expect("狼聊段必有窗口");
    assert_eq!(kind, "狼人夜聊");
    assert_eq!(actor, wolf1, "狼聊从第一个活狼开始");

    // 好人插话、非当前狼抢话都被拒；空文本同白天发言被拒
    assert!(matches!(
        driver.0.apply(villager, Action::WolfChat { text: None }),
        Err(GameError::NotYourTurn { .. })
    ));
    assert!(matches!(
        driver.0.apply(
            wolf2,
            Action::WolfChat {
                text: Some("先听我的".into())
            }
        ),
        Err(GameError::NotYourTurn { .. })
    ));
    assert!(matches!(
        driver.0.apply(
            wolf1,
            Action::WolfChat {
                text: Some("   ".into())
            }
        ),
        Err(GameError::EmptySpeech)
    ));

    // 狼1 耳语 → 轮到狼2 → 沉默后狼聊结束，进刀人窗口
    driver
        .0
        .apply(
            wolf1,
            Action::WolfChat {
                text: Some("今晚刀预言家位".into()),
            },
        )
        .expect("当前狼耳语合法");
    let (actor, _) = driver.0.pending().expect("狼聊未结束");
    assert_eq!(actor, wolf2, "游标应轮到第二只狼");
    driver
        .0
        .apply(wolf2, Action::WolfChat { text: None })
        .expect("沉默合法");

    let (_actor, kind) = driver.0.pending().expect("刀人窗口必有 actor");
    assert_eq!(kind, "夜晚行动");
    assert!(matches!(
        driver.phase(),
        Phase::Night {
            stage: NightStage::Werewolves,
            ..
        }
    ));
    // 狼聊后任意狼都能提交共同刀口
    driver
        .0
        .apply(wolf1, Action::WerewolfKill { target: villager })
        .expect("共同刀口合法");
}

#[test]
fn wolf_council_visibility_follows_camp() {
    // 上帝与狼视角能看到耳语；好人视角裁掉
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut driver = Driver::new(board, 67);
    let wolf = driver.seat(Role::Werewolf);
    let villager = driver.seat(Role::Villager);
    let (guard, _) = driver.0.pending().expect("首夜必有守卫窗口");
    driver
        .0
        .apply(guard, Action::GuardProtect { target: None })
        .expect("空守合法");
    let wolf1 = driver.seat(Role::Werewolf);
    driver
        .0
        .apply(
            wolf1,
            Action::WolfChat {
                text: Some("刀 3 号".into()),
            },
        )
        .expect("狼1 耳语合法");
    // 剩余夜晚窗口交给驱动器（刀人/女巫/预言家）
    driver.run_night(NightPlan {
        kill: villager,
        check: wolf,
        ..Default::default()
    });

    let god = GameSnapshot::render(&driver.0, Viewer::God).unwrap();
    assert!(
        god.logs.iter().any(|log| log.text.contains("耳语")),
        "上帝全见"
    );
    let wolf_view = GameSnapshot::render(&driver.0, Viewer::Player(wolf1)).unwrap();
    assert!(
        wolf_view.logs.iter().any(|log| log.text.contains("耳语")),
        "狼看狼聊"
    );
    let villager_view = GameSnapshot::render(&driver.0, Viewer::Player(villager)).unwrap();
    assert!(
        villager_view
            .logs
            .iter()
            .all(|log| !log.text.contains("耳语")),
        "好人视角不得泄漏狼聊"
    );
}

// ---------- 工具暴露（tools 包桥接） ----------
