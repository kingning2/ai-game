//! PlayerSpawnSystem —— 开局发牌：洗牌、逐座位 spawn 玩家组件、写开局事件。
//!
//! 洗牌语义与 `GameEngine::new` 一致（同一 [`game_domain::Rng`] 实现、同一调用顺序），
//! 黄金测试保证同 seed 下两侧发牌逐座位一致。

use bevy_ecs::prelude::*;
use game_domain::{NightStage, Phase};
use game_events::GameEvent;

use crate::components::{
    DuelUsed, IdiotRevealed, PlayerBundle, PlayerMark, RoleOf, Seat, SeerChecks,
};
use crate::resources::{EventLog, GamePhase, GameRng, SetupConfig};

/// 开局发牌：按板子角色表洗牌后逐座位 spawn（Bevy Entity 由 Commands 延迟创建）
pub fn player_spawn_system(
    setup: Res<SetupConfig>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
    mut log: ResMut<EventLog>,
) {
    let board = &setup.board;
    let mut roles = board.roles();
    rng.0.shuffle(&mut roles);
    for (i, role) in roles.into_iter().enumerate() {
        commands.spawn(PlayerBundle {
            mark: PlayerMark(i as u8),
            seat: Seat {
                alive: true,
                can_vote: true,
            },
            role: RoleOf(role),
            idiot: IdiotRevealed(false),
            duel: DuelUsed(false),
            checks: SeerChecks(Vec::new()),
        });
    }
    commands.insert_resource(GamePhase(Phase::Night {
        day: 1,
        stage: NightStage::Guard,
    }));
    // 开局事件与引擎逐字一致（GameStarted 的 board 计数过滤零人数角色）
    log.events.push(GameEvent::GameStarted {
        players: board.total(),
        board: board
            .counts()
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .map(|(name, count)| (name.to_string(), count))
            .collect(),
    });
}
