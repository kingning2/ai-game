//! 玩家组件：身份 / 座位 / 角色 / 技能痕迹。
//!
//! 对应 `werewolf::player::PlayerState` 字段拆分：
//! `id → PlayerMark`、`alive + can_vote → Seat`、`role → RoleOf`、
//! `revealed_idiot → IdiotRevealed`、`duel_used → DuelUsed`、`checks → SeerChecks`。

use bevy_ecs::prelude::*;
use game_domain::{CheckRecord, PlayerId, Role};

/// 玩家业务身份（对外 ID 用它；Bevy Entity 仅 ECS 内部关联，不外泄）
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerMark(pub PlayerId);

/// 座位状态：存活 + 投票权（白痴翻牌后 `can_vote = false`）
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Seat {
    pub alive: bool,
    pub can_vote: bool,
}

/// 发到的角色（身份保密靠快照视角裁剪，不在组件层做）
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleOf(pub Role);

/// 白痴已翻牌（放逐免死只触发一次）
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IdiotRevealed(pub bool);

/// 骑士已用过决斗权
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DuelUsed(pub bool);

/// 预言家的验人记录（挂预言家 Entity 上，省旁表）
#[derive(Component, Debug, Clone, Default)]
pub struct SeerChecks(pub Vec<CheckRecord>);

/// 玩家组件束：开局 spawn 一次成型
#[derive(Bundle, Debug)]
pub struct PlayerBundle {
    pub mark: PlayerMark,
    pub seat: Seat,
    pub role: RoleOf,
    pub idiot: IdiotRevealed,
    pub duel: DuelUsed,
    pub checks: SeerChecks,
}
