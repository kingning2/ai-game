//! 平民档案

use super::profile::RoleProfile;
use super::{Camp, Role};

/// 平民角色档案
pub const VILLAGER: RoleProfile = RoleProfile {
    role: Role::Villager,
    camp: Camp::Good,
    name: "平民",
    emoji: "👤",
    description: "你是平民，没有任何主动技能。你的武器只有发言、倾听与投票——用逻辑找出狼人。",
    abilities: &["白天发言与投票放逐嫌疑人"],
    constraints: &["没有夜晚技能，夜晚只能等待天亮"],
    night_action: None,
    win_condition: "放逐所有狼人（好人阵营胜利）",
};
