//! 猎人档案

use super::profile::RoleProfile;
use super::{Camp, Role};

/// 猎人角色档案
pub const HUNTER: RoleProfile = RoleProfile {
    role: Role::Hunter,
    camp: Camp::Good,
    name: "猎人",
    emoji: "🏹",
    description: "你是猎人，好人阵营的神职。出局时可以开枪带走一名玩家，是好人阵营的反制底牌。",
    abilities: &["被狼刀杀死或被投票放逐出局时，可开枪带走一名玩家"],
    constraints: &[
        "被女巫毒死时不能开枪（死因是毒药则枪械失效）",
        "枪杀目标必须是活人，也可以选择不开枪",
        "枪死者若也是猎人会级联触发新的开枪窗口",
    ],
    night_action: None,
    win_condition: "放逐所有狼人（好人阵营胜利）",
};
