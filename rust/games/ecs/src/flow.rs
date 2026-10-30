//! 流程助手 —— 引擎控制流的 ECS 版本（查人 / 结算 / 昼夜切换 / 校验）。
//!
//! 与 `werewolf::engine` 对应函数逐行对齐，迁移期以黄金模糊测试守护：
//! 改这里任何一行前先确认 `tests/golden_full.rs` 的期望值来源。

use bevy_ecs::prelude::*;
use game_domain::{
    Camp, CheckResult, DeathCause, GameError, NightStage, Outcome, Phase, PlayerId, PlayerRole,
    Role, WerewolfReason,
};
use game_events::GameEvent;

use crate::components::{DuelUsed, PlayerMark, RoleOf, Seat};
use crate::resources::{
    DuelOffered, EventLog, GamePhase, SetupConfig, WolfChat, WolfCursor,
};

/// 存活计数：(狼数, 神职数, 平民数)
pub(crate) fn counts(world: &mut World) -> (u8, u8, u8) {
    let mut q = world.query::<(&RoleOf, &Seat)>();
    q.iter(world)
        .filter(|(_, seat)| seat.alive)
        .fold((0, 0, 0), |(w, g, v), (role, _)| match role.0.camp() {
            Camp::Werewolf => (w + 1, g, v),
            Camp::Good if role.0.is_god() => (w, g + 1, v),
            Camp::Good => (w, g, v + 1),
        })
}

/// 某角色的唯一存活玩家（夜晚窗口的 actor）；已死则窗口应被跳过
pub(crate) fn find_role(world: &mut World, role: Role) -> Option<PlayerId> {
    let mut q = world.query::<(&PlayerMark, &RoleOf, &Seat)>();
    q.iter(world)
        .find(|(_, r, seat)| r.0 == role && seat.alive)
        .map(|(m, _, _)| m.0)
}

/// 玩家角色（座位必存在：发牌后实体不销毁）
// ponytail: 座位实体发牌后不销毁，find 失败即不变量破坏；
// 对外校验路径已由 check_target_alive/check_night_actor 拦截
#[allow(clippy::expect_used)]
pub(crate) fn role_of(world: &mut World, id: PlayerId) -> Role {
    let mut q = world.query::<(&PlayerMark, &RoleOf)>();
    q.iter(world)
        .find(|(m, _)| m.0 == id)
        .map(|(_, r)| r.0)
        .expect("seat entity must exist")
}

/// 是否存活
pub(crate) fn alive(world: &mut World, id: PlayerId) -> bool {
    let mut q = world.query::<(&PlayerMark, &Seat)>();
    q.iter(world)
        .find(|(m, _)| m.0 == id)
        .is_some_and(|(_, seat)| seat.alive)
}

/// 是否还有本轮投票权
pub(crate) fn can_vote(world: &mut World, id: PlayerId) -> bool {
    let mut q = world.query::<(&PlayerMark, &Seat)>();
    q.iter(world)
        .find(|(m, _)| m.0 == id)
        .is_some_and(|(_, seat)| seat.can_vote)
}

/// 骑士是否已用过决斗权
pub(crate) fn duel_used(world: &mut World, id: PlayerId) -> bool {
    let mut q = world.query::<(&PlayerMark, &DuelUsed)>();
    q.iter(world)
        .find(|(m, _)| m.0 == id)
        .is_some_and(|(_, duel)| duel.0)
}

/// 上帝视角点名
pub(crate) fn named(world: &mut World, id: PlayerId) -> PlayerRole {
    PlayerRole {
        id,
        role: role_of(world, id),
    }
}

/// 座位数
pub(crate) fn total(world: &mut World) -> u8 {
    world.resource::<SetupConfig>().board.total()
}

/// 置死
pub(crate) fn set_dead(world: &mut World, id: PlayerId) {
    let mut q = world.query::<(&PlayerMark, &mut Seat)>();
    for (m, mut seat) in q.iter_mut(world) {
        if m.0 == id {
            seat.alive = false;
        }
    }
}

/// 设置投票权
pub(crate) fn set_can_vote(world: &mut World, id: PlayerId, value: bool) {
    let mut q = world.query::<(&PlayerMark, &mut Seat)>();
    for (m, mut seat) in q.iter_mut(world) {
        if m.0 == id {
            seat.can_vote = value;
        }
    }
}

/// 标记骑士决斗权已用
pub(crate) fn set_duel_used(world: &mut World, id: PlayerId) {
    let mut q = world.query::<(&PlayerMark, &mut DuelUsed)>();
    for (m, mut duel) in q.iter_mut(world) {
        if m.0 == id {
            duel.0 = true;
        }
    }
}

/// 胜负判定（屠边规则）。分出胜负则广播事件并置为 Settled
pub(crate) fn check_outcome(world: &mut World) -> bool {
    let (wolves, gods, villagers) = counts(world);
    let outcome = if wolves == 0 {
        Some(Outcome::GoodWin)
    } else if gods == 0 {
        // 屠边：神职全灭
        Some(Outcome::WerewolfWin {
            reason: WerewolfReason::GodsWiped,
        })
    } else if villagers == 0 {
        // 屠边：平民全灭
        Some(Outcome::WerewolfWin {
            reason: WerewolfReason::VillagersWiped,
        })
    } else {
        None
    };
    match outcome {
        Some(outcome) => {
            world
                .resource_mut::<EventLog>()
                .events
                .push(GameEvent::GameOver {
                    outcome: outcome.clone(),
                });
            world.insert_resource(GamePhase(Phase::Settled { outcome }));
            true
        }
        None => false,
    }
}

/// 白天统一击杀：置死 + 翻牌广播
pub(crate) fn kill(world: &mut World, target: PlayerId, day: u32, cause: DeathCause) {
    set_dead(world, target);
    let player = named(world, target);
    world
        .resource_mut::<EventLog>()
        .events
        .push(GameEvent::PlayerDied { day, player, cause });
}

/// 从 `from` 座位起（含）找下一个活人座位
pub(crate) fn next_alive_speaker(world: &mut World, from: PlayerId) -> Option<PlayerId> {
    let total = total(world);
    (0..total)
        .map(|offset| (from + offset) % total)
        .find(|seat| alive(world, *seat))
}

/// 今天已发言的座位数（从事件流统计）
pub(crate) fn spoken_today(world: &mut World, day: u32) -> usize {
    world
        .resource::<EventLog>()
        .events
        .iter()
        .filter(|event| {
            matches!(
                event,
                GameEvent::SpeechMade {
                    day: d,
                    ..
                } if *d == day
            )
        })
        .count()
}

/// 入昼：从首个死者下一位开始发言（平安夜从 0 号开始）
pub(crate) fn begin_day(world: &mut World, day: u32) {
    let first_death = world
        .resource::<EventLog>()
        .events
        .iter()
        .rev()
        .find_map(|event| match event {
            GameEvent::Dawn {
                day: d,
                deaths,
            } if *d == day && !deaths.is_empty() => Some(deaths[0].player),
            _ => None,
        });
    let total = total(world);
    let next_speaker = match first_death {
        Some(dead) => (dead + 1) % total,
        None => 0,
    };
    world.insert_resource(GamePhase(Phase::DaySpeech { day, next_speaker }));
    world.insert_resource(DuelOffered(false));
}

/// 进入新夜晚：广播天黑，并把起始窗口定位到第一个存活角色
/// （板子没有守卫 / 该角色昨夜阵亡都会被跳过）
pub(crate) fn enter_night(world: &mut World, day: u32) {
    world
        .resource_mut::<EventLog>()
        .events
        .push(GameEvent::NightBegan { day });
    skip_dead_stages(world, NightStage::Guard, day);
}

/// 从 `stage` 起找到第一个「存活角色」的窗口；后面全无存活角色则直接结算
/// （理论不可达：好人全灭时胜负早已判定）
pub(crate) fn skip_dead_stages(world: &mut World, stage: NightStage, day: u32) {
    let mut stage = stage;
    loop {
        if find_role(world, stage.role()).is_some() {
            world.insert_resource(GamePhase(Phase::Night { day, stage }));
            if stage == NightStage::WerewolfChat {
                // open_wolf_chat：游标定位到第一个活狼（该段能进来就必有活狼）
                // ponytail: skip_dead_stages 已确认 stage.role() 有存活者，不变量破坏即 panic 合理
                #[allow(clippy::expect_used)]
                let start = find_role(world, Role::Werewolf)
                    .expect("werewolf chat stage implies a living wolf");
                world.insert_resource(WolfChat(Some(WolfCursor {
                    cursor: start,
                    start,
                })));
            }
            return;
        }
        match stage.next() {
            Some(next) => stage = next,
            None => {
                crate::night::resolve_night(world, day);
                return;
            }
        }
    }
}

// ---------- 校验工具 ----------

/// 夜晚窗口的角色校验：actor 必须是该角色的唯一存活者
pub(crate) fn check_night_actor(
    world: &mut World,
    actor: PlayerId,
    role: Role,
) -> Result<(), GameError> {
    match find_role(world, role) {
        Some(expected) if expected == actor => Ok(()),
        Some(expected) => Err(GameError::NotYourTurn {
            expected,
            actual: actor,
        }),
        None => Err(GameError::WrongPhase("该角色窗口因出局已跳过")),
    }
}

pub(crate) fn check_target_alive(world: &mut World, target: PlayerId) -> Result<(), GameError> {
    GameError::check_seat(target, total(world))?;
    if !alive(world, target) {
        return Err(GameError::TargetDead(target));
    }
    Ok(())
}

/// 狼队共享窗口：任意存活狼都能提交刀口（区别于 [`check_night_actor`] 的唯一者校验）
pub(crate) fn check_wolf_actor(world: &mut World, actor: PlayerId) -> Result<(), GameError> {
    let is_wolf = alive(world, actor) && role_of(world, actor) == Role::Werewolf;
    if !is_wolf {
        let expected = find_role(world, Role::Werewolf).unwrap_or(actor);
        return Err(GameError::NotYourTurn {
            expected,
            actual: actor,
        });
    }
    Ok(())
}

/// 预言家验人结果（狼/好人）
pub(crate) fn check_result(role: Role) -> CheckResult {
    match role.camp() {
        Camp::Werewolf => CheckResult::Werewolf,
        Camp::Good => CheckResult::Good,
    }
}
