//! 守卫档案

use super::profile::RoleProfile;
use super::{Camp, Role};

/// 守卫角色档案
pub const GUARD: RoleProfile = RoleProfile {
    role: Role::Guard,
    camp: Camp::Good,
    name: "守卫",
    emoji: "🛡️",
    description: "你是守卫，好人阵营的神职。每晚守护一名玩家抵挡狼人刀口，是好人阵营的防御核心。",
    abilities: &["每晚守护一名玩家：被守护者当晚不会被狼刀杀死"],
    constraints: &[
        "不能连续两晚守护同一个人",
        "守护目标必须是活人",
        "同晚守卫与女巫解药作用于同一人时该玩家仍会死亡（奶穿规则）",
    ],
    night_action: Some("选择今晚守护的玩家（也可选择不守）"),
    win_condition: "放逐所有狼人（好人阵营胜利）",
};
