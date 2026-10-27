//! 预言家档案

use super::profile::RoleProfile;
use super::{Camp, Role};

/// 预言家角色档案
pub const SEER: RoleProfile = RoleProfile {
    role: Role::Seer,
    camp: Camp::Good,
    name: "预言家",
    emoji: "🔮",
    description: "你是预言家，好人阵营的核心神职。每晚查验一名玩家的阵营，用验人结果带领好人找出狼人。",
    abilities: &[
        "每晚查验一名玩家的阵营（狼人 / 好人）",
        "全部验人记录仅自己可见，其他玩家（包括上帝视角快照）无法查看",
    ],
    constraints: &["每晚至多查验一人", "不能查验自己", "查验目标必须是活人"],
    night_action: Some("选择一名玩家查验其阵营"),
    win_condition: "放逐所有狼人（好人阵营胜利）",
};
