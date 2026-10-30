//! 白天阶段 —— 发言 / 骑士决斗 / 投票放逐 / 猎人开枪。
//!
//! 与 `werewolf::engine` 的 `apply_speech` / `apply_duel` / `apply_vote` /
//! `apply_hunter_shot` 等逐行对齐，迁移期以黄金模糊测试守护。

use bevy_ecs::prelude::*;
use game_domain::{
    Action, Camp, DeathCause, GameError, Phase, PlayerId, PlayerRole, Role,
};
use game_events::GameEvent;

use crate::components::{IdiotRevealed, PlayerMark, Seat};
use crate::flow::{
    alive, can_vote, check_outcome, check_target_alive, duel_used, enter_night, find_role, kill,
    named, next_alive_speaker, role_of, set_can_vote, set_duel_used, spoken_today, total,
};
use crate::resources::{DuelOffered, EventLog, GamePhase, Votes};

// ---------- 发言 ----------

pub(crate) fn apply_speech(
    world: &mut World,
    actor: PlayerId,
    action: Action,
    day: u32,
    next_speaker: PlayerId,
) -> Result<(), GameError> {
    let expected = next_alive_speaker(world, next_speaker)
        .ok_or(GameError::WrongPhase("无存活发言者"))?;
    if actor != expected {
        return Err(GameError::NotSpeakerTurn { expected, actual: actor });
    }
    let Action::Speech { text } = action else {
        return Err(GameError::WrongPhase("发言窗口只接受 Speech"));
    };
    if text.trim().is_empty() {
        return Err(GameError::EmptySpeech);
    }
    world
        .resource_mut::<EventLog>()
        .events
        .push(GameEvent::SpeechMade { day, speaker: actor, text });
    advance_speech(world, day, actor);
    Ok(())
}

/// 发言轮转：今天已发言数 < 存活数 ⇒ 下一位；否则进入决斗/投票
fn advance_speech(world: &mut World, day: u32, just_spoke: PlayerId) {
    let total = total(world);
    let alive_count = (0..total).filter(|seat| alive(world, *seat)).count();
    let spoken = spoken_today(world, day);
    if spoken < alive_count {
        // 不用 expect：找不到下一位（理论上不可能，防御式处理）直接推进到投票阶段
        if let Some(next) = next_alive_speaker(world, (just_spoke + 1) % total) {
            world.insert_resource(GamePhase(Phase::DaySpeech {
                day,
                next_speaker: next,
            }));
            return;
        }
    }
    after_speech(world, day);
}

/// 发言结束 → 骑士决斗窗口（存活且未用过）→ 投票
fn after_speech(world: &mut World, day: u32) {
    if !world.resource::<DuelOffered>().0
        && let Some(knight) = find_role(world, Role::Knight)
        && !duel_used(world, knight)
    {
        world.insert_resource(DuelOffered(true));
        world.insert_resource(GamePhase(Phase::KnightDuel { day, knight }));
        return;
    }
    world.insert_resource(GamePhase(Phase::DayVote { day }));
}

// ---------- 骑士决斗 ----------

pub(crate) fn apply_duel(
    world: &mut World,
    actor: PlayerId,
    action: Action,
    day: u32,
    knight: PlayerId,
) -> Result<(), GameError> {
    if actor != knight {
        return Err(GameError::NotYourTurn {
            expected: knight,
            actual: actor,
        });
    }
    let Action::KnightDuel { target } = action else {
        return Err(GameError::WrongPhase("决斗窗口只接受 KnightDuel"));
    };
    set_duel_used(world, knight);
    if let Some(target) = target {
        check_target_alive(world, target)?;
        let target_is_wolf = role_of(world, target).camp() == Camp::Werewolf;
        let loser = if target_is_wolf {
            named(world, target)
        } else {
            // 骑士赌输：骑士死（决斗死亡不触发猎人开枪）
            named(world, knight)
        };
        let loser_id = loser.id;
        kill(world, loser_id, day, DeathCause::Duel);
        world
            .resource_mut::<EventLog>()
            .events
            .push(GameEvent::KnightDuelResolved {
                day,
                knight,
                target,
                target_is_wolf,
                loser,
            });
        if check_outcome(world) {
            return Ok(());
        }
    }
    world.insert_resource(GamePhase(Phase::DayVote { day }));
    Ok(())
}

// ---------- 投票 ----------

pub(crate) fn apply_vote(
    world: &mut World,
    actor: PlayerId,
    action: Action,
    day: u32,
) -> Result<(), GameError> {
    let Action::Vote { target } = action else {
        return Err(GameError::WrongPhase("投票窗口只接受 Vote"));
    };
    // 投票不强制顺序：任何未投票的活人都可以投；已投/已死/无权 → 拒绝
    if !alive(world, actor) {
        return Err(GameError::TargetDead(actor));
    }
    if !can_vote(world, actor) {
        return Err(GameError::AlreadyVoted(actor));
    }
    if let Some(target) = target {
        check_target_alive(world, target)?;
    }
    world.resource_mut::<Votes>().0.push(target);
    set_can_vote(world, actor, false);

    // 还有未投票的活人吗？
    let mut q = world.query::<&Seat>();
    if q.iter(world).any(|seat| seat.alive && seat.can_vote) {
        return Ok(());
    }
    resolve_vote(world, day);
    Ok(())
}

/// 结算投票：统计 → 平票无人放逐 / 放逐（白痴翻牌 / 猎人窗口）→ 胜负 → 入夜
fn resolve_vote(world: &mut World, day: u32) {
    let votes = std::mem::take(&mut world.resource_mut::<Votes>().0);
    let mut tally: Vec<(PlayerId, u8)> = Vec::new();
    for vote in votes.iter().flatten() {
        match tally.iter_mut().find(|(seat, _)| seat == vote) {
            Some(entry) => entry.1 += 1,
            None => tally.push((*vote, 1)),
        }
    }

    let max = tally.iter().map(|(_, count)| *count).max().unwrap_or(0);
    let top: Vec<PlayerId> = tally
        .iter()
        .filter(|(_, count)| *count == max && max > 0)
        .map(|(seat, _)| *seat)
        .collect();

    // 恢复所有活人投票权（白痴除外，下面对其单独处理）
    let mut q = world.query::<&mut Seat>();
    for mut seat in q.iter_mut(world) {
        if seat.alive {
            seat.can_vote = true;
        }
    }

    if top.len() != 1 {
        world
            .resource_mut::<EventLog>()
            .events
            .push(GameEvent::VoteResolved {
                day,
                counts: tally,
                tie: true,
                exiled: None,
            });
        check_outcome(world);
        enter_night(world, day + 1);
        return;
    }

    let exiled = top[0];
    let exiled_named = named(world, exiled);
    world
        .resource_mut::<EventLog>()
        .events
        .push(GameEvent::VoteResolved {
            day,
            counts: tally,
            tie: false,
            exiled: Some(exiled_named),
        });

    // 白痴翻牌：放逐免死、失去投票权（revealed_idiot 标记）
    if role_of(world, exiled) == Role::Idiot {
        set_can_vote(world, exiled, false);
        set_idiot_revealed(world, exiled);
        world
            .resource_mut::<EventLog>()
            .events
            .push(GameEvent::IdiotRevealed { day, idiot: exiled });
        check_outcome(world);
        enter_night(world, day + 1);
        return;
    }

    kill(world, exiled, day, DeathCause::Exiled);
    if check_outcome(world) {
        return;
    }
    // 被放逐猎人可开枪
    if role_of(world, exiled) == Role::Hunter {
        world.insert_resource(GamePhase(Phase::HunterShot { day, hunter: exiled }));
        return;
    }
    enter_night(world, day + 1);
}

/// 标记白痴已翻牌
fn set_idiot_revealed(world: &mut World, id: PlayerId) {
    let mut q = world.query::<(&PlayerMark, &mut IdiotRevealed)>();
    for (m, mut idiot) in q.iter_mut(world) {
        if m.0 == id {
            idiot.0 = true;
        }
    }
}

// ---------- 猎人开枪 ----------

pub(crate) fn apply_hunter_shot(
    world: &mut World,
    actor: PlayerId,
    action: Action,
    day: u32,
    hunter: PlayerId,
) -> Result<(), GameError> {
    if actor != hunter {
        return Err(GameError::NotYourTurn {
            expected: hunter,
            actual: actor,
        });
    }
    let Action::HunterShot { target } = action else {
        return Err(GameError::WrongPhase("猎人窗口只接受 HunterShot"));
    };
    let hunter_named = named(world, hunter);
    let mut shot: Option<PlayerRole> = None;
    if let Some(target) = target {
        check_target_alive(world, target)?;
        kill(world, target, day, DeathCause::Shot);
        shot = Some(named(world, target));
    }
    world
        .resource_mut::<EventLog>()
        .events
        .push(GameEvent::HunterShotFired {
            day,
            hunter: hunter_named,
            target: shot,
        });
    if check_outcome(world) {
        return Ok(());
    }
    // 级联：枪死者也是猎人 ⇒ 新窗口
    if let Some(target) = target
        && role_of(world, target) == Role::Hunter
    {
        world.insert_resource(GamePhase(Phase::HunterShot { day, hunter: target }));
        return Ok(());
    }
    // 窗口去向：放逐触发 ⇒ 下一夜；夜晚触发 ⇒ 回到白天发言
    let vote_done = world
        .resource::<EventLog>()
        .events
        .iter()
        .any(|event| matches!(event, GameEvent::VoteResolved { day: d, .. } if *d == day));
    if vote_done {
        enter_night(world, day + 1);
    } else {
        crate::flow::begin_day(world, day);
    }
    Ok(())
}
