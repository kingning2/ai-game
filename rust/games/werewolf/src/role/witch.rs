//! 女巫档案

use super::profile::RoleProfile;
use super::{Camp, Role};

/// 女巫角色档案
pub const WITCH: RoleProfile = RoleProfile {
    role: Role::Witch,
    camp: Camp::Good,
    name: "女巫",
    emoji: "🧪",
    description: "你是女巫，好人阵营的神职。手中有一瓶解药和一瓶毒药，两瓶药各自全程只能使用一次。",
    abilities: &[
        "解药：救活当晚被刀杀的玩家（一局一次）",
        "毒药：毒杀一名玩家（一局一次）",
        "夜晚会被告知今晚的刀口是谁（决定是否用解药）",
    ],
    constraints: &[
        "解药与毒药各只有一瓶，用完即失",
        "同一晚至多使用一瓶药，不能既救人又毒人",
        "今晚没有刀口时不能使用解药（空刀夜不能空救）",
        "毒药目标必须是活人，且不能毒自己",
    ],
    night_action: Some("决定是否用解药救当晚刀口、是否用毒药毒人"),
    win_condition: "放逐所有狼人（好人阵营胜利）",
};
