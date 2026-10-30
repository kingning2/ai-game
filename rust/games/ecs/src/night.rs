//! 夜晚阶段 —— 守卫 / 狼聊 / 刀人 / 女巫 / 预言家 + 夜间结算。
//!
//! 与 `werewolf::engine` 的 `apply_night` / `resolve_night` 等逐行对齐，
//! 迁移期以黄金模糊测试守护。

use bevy_ecs::prelude::*;
use game_domain::{
    Action, CheckRecord, DeathCause, GameError, NightStage, Phase, PlayerId, Role,
};
use game_events::{GameEvent, NightDeath};

use crate::components::{PlayerMark, SeerChecks};
use crate::flow::{
    alive, check_night_actor, check_result, check_target_alive, check_wolf_actor, check_outcome,
    named, role_of, skip_dead_stages, begin_day, total,
};
use crate::resources::{
    EventLog, GamePhase, LastProtected, NightRecord, Potions, WolfChat, WolfCursor,
};

/// 夜晚窗口动作入口（与引擎 `apply_night` 对齐）
pub(crate) fn apply_night(
    world: &mut World,
    actor: PlayerId,
    action: Action,
    day: u32,
    stage: NightStage,
) -> Result<(), GameError> {
    let logged = action.clone();
    match (stage, action) {
        (NightStage::Guard, Action::GuardProtect { target }) => {
            check_night_actor(world, actor, Role::Guard)?;
            apply_guard(world, target)?;
        }
        (NightStage::WerewolfChat, action) => {
            let Action::WolfChat { text } = action else {
                return Err(GameError::WrongPhase("狼聊窗口只接受 WolfChat"));
            };
            // 狼聊自带专用事件与推进（游标绕回起点 → skip_dead_stages 进刀人窗口），
            // 不走尾部的统一 NightAction 上报与 advance_night
            apply_wolf_chat(world, actor, text, day)?;
            return Ok(());
        }
        (NightStage::Werewolves, Action::WerewolfKill { target }) => {
            check_wolf_actor(world, actor)?;
            check_target_alive(world, target)?;
            world.resource_mut::<NightRecord>().killed = Some(target);
        }
        (NightStage::Witch, Action::WitchAct { save, poison }) => {
            check_night_actor(world, actor, Role::Witch)?;
            apply_witch(world, actor, save, poison)?;
        }
        (NightStage::Seer, Action::SeerCheck { target }) => {
            check_night_actor(world, actor, Role::Seer)?;
            apply_seer(world, actor, target, day)?;
        }
        _ => return Err(GameError::WrongPhase("夜晚动作与当前窗口不符")),
    }
    let speaker = named(world, actor);
    world
        .resource_mut::<EventLog>()
        .events
        .push(GameEvent::NightAction {
            day,
            actor: speaker,
            action: logged,
        });
    advance_night(world, day);
    Ok(())
}

/// 守卫守护（`None` = 空守）；不能与昨晚守同一人
fn apply_guard(world: &mut World, target: Option<PlayerId>) -> Result<(), GameError> {
    if let Some(target) = target {
        check_target_alive(world, target)?;
        if world.resource::<LastProtected>().0 == Some(target) {
            return Err(GameError::GuardRepeat);
        }
        world.resource_mut::<NightRecord>().protected = Some(target);
    }
    Ok(())
}

/// 女巫行动：同晚至多用一瓶；解药需有刀口；毒药不能指向自己
fn apply_witch(
    world: &mut World,
    witch: PlayerId,
    save: bool,
    poison: Option<PlayerId>,
) -> Result<(), GameError> {
    if save && poison.is_some() {
        return Err(GameError::WitchBothPotions);
    }
    if save {
        if !world.resource::<Potions>().save {
            return Err(GameError::PotionDepleted("解药"));
        }
        let kill = world
            .resource::<NightRecord>()
            .killed
            .ok_or(GameError::NoKillToSave)?;
        check_target_alive(world, kill)?;
        world.resource_mut::<NightRecord>().saved = true;
        world.resource_mut::<Potions>().save = false;
    }
    if let Some(target) = poison {
        check_target_alive(world, target)?;
        if !world.resource::<Potions>().poison {
            return Err(GameError::PotionDepleted("毒药"));
        }
        if target == witch {
            return Err(GameError::SelfTarget("女巫"));
        }
        world.resource_mut::<NightRecord>().poisoned = Some(target);
        world.resource_mut::<Potions>().poison = false;
    }
    Ok(())
}

/// 预言家验人：记录挂在预言家 Entity 的 `SeerChecks` 上
fn apply_seer(
    world: &mut World,
    seer: PlayerId,
    target: PlayerId,
    day: u32,
) -> Result<(), GameError> {
    check_target_alive(world, target)?;
    if target == seer {
        return Err(GameError::SelfTarget("预言家"));
    }
    let result = check_result(role_of(world, target));
    let record = CheckRecord {
        day,
        target,
        result,
    };
    let mut q = world.query::<(&PlayerMark, &mut SeerChecks)>();
    for (m, mut checks) in q.iter_mut(world) {
        if m.0 == seer {
            checks.0.push(record.clone());
        }
    }
    Ok(())
}

/// 狼队夜聊：当前游标狼发言/沉默，然后轮到下一位活狼；
/// 绕回起点则夜聊结束，进入刀人窗口
fn apply_wolf_chat(
    world: &mut World,
    actor: PlayerId,
    text: Option<String>,
    day: u32,
) -> Result<(), GameError> {
    let chat = match world.resource::<WolfChat>().0 {
        Some(chat) if chat.cursor == actor => chat,
        Some(chat) => {
            return Err(GameError::NotYourTurn {
                expected: chat.cursor,
                actual: actor,
            });
        }
        None => return Err(GameError::WrongPhase("狼聊窗口未开启")),
    };
    let speaker = named(world, actor);
    if let Some(text) = text {
        let text = text.trim();
        if text.is_empty() {
            return Err(GameError::EmptySpeech);
        }
        world
            .resource_mut::<EventLog>()
            .events
            .push(GameEvent::WolfChatSpoke {
                day,
                speaker,
                text: text.to_string(),
            });
    } else {
        world
            .resource_mut::<EventLog>()
            .events
            .push(GameEvent::WolfChatPassed { day, speaker });
    }
    // 下一位活狼（只在狼之间轮转）；绕回起点（含单狼自轮）则夜聊结束，进刀人窗口
    let total = total(world);
    let next = (0..total)
        .map(|offset| (chat.cursor + 1 + offset) % total)
        .find(|seat| alive(world, *seat) && role_of(world, *seat) == Role::Werewolf)
        .unwrap_or(chat.start);
    if next == chat.start {
        world.resource_mut::<WolfChat>().0 = None;
        skip_dead_stages(world, NightStage::Werewolves, day);
    } else {
        world.resource_mut::<WolfChat>().0 = Some(WolfCursor {
            cursor: next,
            start: chat.start,
        });
    }
    Ok(())
}

/// 夜晚推进：当前窗口动作已收 → 下一段（跳过无存活角色的段）；最后一段 → 结算
fn advance_night(world: &mut World, day: u32) {
    let stage = match &world.resource::<GamePhase>().0 {
        Phase::Night { stage, .. } => *stage,
        _ => return,
    };
    match stage.next() {
        Some(next) => skip_dead_stages(world, next, day),
        None => resolve_night(world, day),
    }
}

/// 夜晚结算：刀口 + 守护 + 解药 + 毒药 → 死亡名单 → 猎人窗口/入昼
///
/// 规则要点：
/// - 被刀未守未救 ⇒ 死；被刀且被守 ⇒ 平安；被刀且被救 ⇒ 平安；
/// - **奶穿**：被刀且既被守又被救 ⇒ 死（双保险冲突）
/// - 毒杀独立结算，被毒猎人不能开枪
pub(crate) fn resolve_night(world: &mut World, day: u32) {
    let record = *world.resource::<NightRecord>();
    let mut deaths: Vec<NightDeath> = Vec::new();

    if let Some(kill) = record.killed {
        let guarded = record.protected == Some(kill);
        // (false,false) 刀死；(true,true) 奶穿死；其余平安
        if guarded == record.saved {
            deaths.push(NightDeath {
                player: kill,
                cause: DeathCause::Slain,
            });
        }
    }
    if let Some(poison) = record.poisoned
        && !deaths.iter().any(|d| d.player == poison)
    {
        deaths.push(NightDeath {
            player: poison,
            cause: DeathCause::Poisoned,
        });
    }

    world.insert_resource(LastProtected(record.protected));
    world.insert_resource(NightRecord::default());

    // 被刀死的猎人可开枪（先收集，后击杀）
    let hunter_shots: Vec<PlayerId> = deaths
        .iter()
        .filter(|d| d.cause == DeathCause::Slain)
        .map(|d| d.player)
        .filter(|id| role_of(world, *id) == Role::Hunter)
        .collect();

    for death in &deaths {
        crate::flow::set_dead(world, death.player);
    }
    for death in &deaths {
        let player = named(world, death.player);
        world
            .resource_mut::<EventLog>()
            .events
            .push(GameEvent::PlayerDied {
                day,
                player,
                cause: death.cause,
            });
    }

    let game_over = check_outcome(world);
    world
        .resource_mut::<EventLog>()
        .events
        .push(GameEvent::Dawn { day, deaths });
    if game_over {
        return;
    }
    match hunter_shots.first() {
        Some(hunter) => {
            world.insert_resource(GamePhase(Phase::HunterShot {
                day,
                hunter: *hunter,
            }));
        }
        None => begin_day(world, day),
    }
}
