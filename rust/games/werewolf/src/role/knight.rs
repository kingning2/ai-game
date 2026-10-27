//! 骑士档案

use super::profile::RoleProfile;
use super::{Camp, Role};

/// 骑士角色档案
pub const KNIGHT: RoleProfile = RoleProfile {
    role: Role::Knight,
    camp: Camp::Good,
    name: "骑士",
    emoji: "⚔️",
    description: "你是骑士，好人阵营的神职。白天可翻牌决斗一名玩家：对方是狼人则狼死，否则你以死谢罪。",
    abilities: &[
        "白天发言阶段可发起决斗：翻牌亮明身份并指定一名玩家",
        "决斗目标被验证为狼人时，狼人当场出局",
    ],
    constraints: &[
        "一局只能决斗一次",
        "决斗目标被验证为好人时，骑士当场出局（赌输了以死谢罪）",
        "决斗只能在白天决斗窗口发起，可以跳过",
    ],
    night_action: None,
    win_condition: "放逐所有狼人（好人阵营胜利）",
};
