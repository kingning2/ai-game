//! 动作入口 —— ECS 版 [`GameEngine::apply`]：校验 → 推进 → 返回增量事件。
//!
//! 错误语义与引擎一致：校验失败时不留下任何半截事件（日志回滚到入口长度）。

use bevy_ecs::prelude::*;
use game_domain::{Action, GameError, Phase, PlayerId};
use game_events::GameEvent;

use crate::day;
use crate::night;
use crate::resources::{EventLog, GamePhase};

/// 喂入一个动作，推进状态机，返回本次产生的增量事件。
pub fn apply_action(
    world: &mut World,
    actor: PlayerId,
    action: Action,
) -> Result<Vec<GameEvent>, GameError> {
    if matches!(world.resource::<GamePhase>().0, Phase::Settled { .. }) {
        return Err(GameError::GameOver);
    }
    let before = world.resource::<EventLog>().events.len();
    let phase = world.resource::<GamePhase>().0.clone();
    let result = match phase {
        Phase::Night { day, stage } => night::apply_night(world, actor, action, day, stage),
        Phase::DaySpeech { day, next_speaker } => {
            day::apply_speech(world, actor, action, day, next_speaker)
        }
        Phase::KnightDuel { day, knight } => day::apply_duel(world, actor, action, day, knight),
        Phase::DayVote { day } => day::apply_vote(world, actor, action, day),
        Phase::HunterShot { day, hunter } => day::apply_hunter_shot(world, actor, action, day, hunter),
        Phase::Settled { .. } => Err(GameError::GameOver),
    };
    match result {
        Ok(()) => Ok(world.resource::<EventLog>().events[before..].to_vec()),
        Err(err) => {
            world.resource_mut::<EventLog>().events.truncate(before);
            Err(err)
        }
    }
}
