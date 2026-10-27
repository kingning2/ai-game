//! 狼人档案

use super::profile::RoleProfile;
use super::{Camp, Role};

/// 狼人角色档案
pub const WEREWOLF: RoleProfile = RoleProfile {
    role: Role::Werewolf,
    camp: Camp::Werewolf,
    name: "狼人",
    emoji: "🐺",
    description: "你是狼人。白天伪装成好人混入发言与投票，夜晚与狼队共同猎杀一名玩家。",
    abilities: &[
        "夜晚与所有存活狼人共同决定刀杀一名玩家（任意存活狼人都可提交刀口）",
        "开局即知晓全部狼队友的座位与身份",
    ],
    constraints: &[
        "狼队每晚共享一次刀口，意见不一致时需在狼队内协调",
        "刀口目标必须是活人",
    ],
    night_action: Some("与狼队共同选择今晚的刀杀目标"),
    win_condition: "屠边获胜：场上神职全灭（屠神）或平民全灭（屠民）",
};
