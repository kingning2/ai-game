//! 白痴档案

use super::profile::RoleProfile;
use super::{Camp, Role};

/// 白痴角色档案
pub const IDIOT: RoleProfile = RoleProfile {
    role: Role::Idiot,
    camp: Camp::Good,
    name: "白痴",
    emoji: "🤡",
    description: "你是白痴，好人阵营的特殊神职。被投票放逐时翻牌亮明身份免死，但从此失去投票权。",
    abilities: &["被投票放逐时翻牌亮明身份，免于出局，继续留在场上"],
    constraints: &[
        "翻牌免死后失去投票权，只能发言不能再投票",
        "夜晚被刀杀或被毒死不触发翻牌，直接出局",
    ],
    night_action: None,
    win_condition: "放逐所有狼人（好人阵营胜利）",
};
