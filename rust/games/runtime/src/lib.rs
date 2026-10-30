//! 狼人杀运行时 —— [`GameManager`]：每局一个 ECS World 的生命周期门面。
//!
//! 对外 API 与旧引擎签名逐字一致（`new/phase/events/settled/total_players/`
//! `players/pending/apply`），server / agent / 快照层零改动切换。
//! `players()` 返回缓存切片：仅在 `new` / `apply` 后重建（全部变更都走这两个入口）。

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use bevy_ecs::prelude::*;
use game_domain::{
    Action, BoardConfig, BoardError, GameError, NightStage, Phase, PlayerId, PlayerState,
};
use game_events::GameEvent;

use game_ecs::resources::{EventLog, GamePhase, WolfChat};
use game_ecs::{apply_action, setup_world};

/// 一局游戏：独立 World + 玩家视图缓存
pub struct GameManager {
    world: World,
    players: Vec<PlayerState>,
}

impl std::fmt::Debug for GameManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameManager")
            .field("phase", self.phase())
            .field("events", &self.events().len())
            .finish_non_exhaustive()
    }
}

impl GameManager {
    /// 开局：校验板子、发牌、开启第一夜。`seed` 相同 ⇒ 发牌相同（可复现）
    pub fn new(board: BoardConfig, seed: u64) -> Result<Self, BoardError> {
        let mut world = World::new();
        setup_world(&mut world, board, seed)?;
        let mut manager = Self {
            world,
            players: Vec::new(),
        };
        manager.refresh_players();
        Ok(manager)
    }

    /// 当前阶段
    pub fn phase(&self) -> &Phase {
        &self.world.resource::<GamePhase>().0
    }

    /// 完整上帝事件日志（seq = 下标，单调连续）
    pub fn events(&self) -> &[GameEvent] {
        &self.world.resource::<EventLog>().events
    }

    pub fn settled(&self) -> bool {
        matches!(self.phase(), Phase::Settled { .. })
    }

    /// 座位数
    pub fn total_players(&self) -> u8 {
        self.players.len() as u8
    }

    /// 玩家状态（按座位序；保密靠快照层裁剪）
    pub fn players(&self) -> &[PlayerState] {
        &self.players
    }

    /// 当前等谁做什么（对齐旧引擎 `GameEngine::pending` 的标签文案）
    pub fn pending(&self) -> Option<(PlayerId, &'static str)> {
        match self.phase() {
            Phase::Night { stage, .. } => match stage {
                NightStage::WerewolfChat => {
                    let cursor = self.world.resource::<WolfChat>().0.as_ref()?;
                    Some((cursor.cursor, "狼人夜聊"))
                }
                _ => {
                    let role = stage.role();
                    self.players
                        .iter()
                        .find(|p| p.role == role && p.alive)
                        .map(|p| (p.id, "夜晚行动"))
                }
            },
            Phase::DaySpeech { next_speaker, .. } => {
                let total = self.total_players();
                (0..total)
                    .map(|offset| (next_speaker + offset) % total)
                    .find(|seat| self.players[*seat as usize].alive)
                    .map(|id| (id, "白天发言"))
            }
            Phase::KnightDuel { knight, .. } => Some((*knight, "骑士决斗")),
            Phase::DayVote { .. } => self
                .players
                .iter()
                .find(|p| p.alive && p.can_vote)
                .map(|p| (p.id, "投票")),
            Phase::HunterShot { hunter, .. } => Some((*hunter, "猎人开枪")),
            Phase::Settled { .. } => None,
        }
    }

    /// 喂入一个动作，推进状态机，返回增量事件流。
    /// 校验失败时不留下任何半截事件（ecs 层已回滚）。
    pub fn apply(&mut self, actor: PlayerId, action: Action) -> Result<Vec<GameEvent>, GameError> {
        let result = apply_action(&mut self.world, actor, action);
        if result.is_ok() {
            self.refresh_players();
        }
        result
    }

    /// 从 ECS 组件重建玩家视图缓存（按座位排序）
    fn refresh_players(&mut self) {
        use game_ecs::components::{DuelUsed, IdiotRevealed, PlayerMark, RoleOf, Seat, SeerChecks};
        let mut query = self
            .world
            .query::<(&PlayerMark, &RoleOf, &Seat, &IdiotRevealed, &DuelUsed, &SeerChecks)>();
        let mut rows: Vec<PlayerState> = query
            .iter(&self.world)
            .map(|(mark, role, seat, idiot, duel, checks)| PlayerState {
                id: mark.0,
                role: role.0,
                alive: seat.alive,
                can_vote: seat.can_vote,
                revealed_idiot: idiot.0,
                duel_used: duel.0,
                checks: checks.0.clone(),
            })
            .collect();
        rows.sort_by_key(|p| p.id);
        self.players = rows;
    }
}
