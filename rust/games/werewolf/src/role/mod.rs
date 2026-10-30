use serde::{Deserialize, Serialize};

pub mod profile;

pub use profile::{RoleProfile, profile};

/// 角色阵营。胜负判定按阵营计数（屠边规则）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Camp {
    /// 狼人阵营：夜晚共同刀人
    Werewolf,
    /// 好人阵营：神职 + 平民
    Good,
}

/// 角色全集（全角色版）。
///
/// 「区分角色」体现在四个维度：
/// 1. [`Role::camp`]：阵营归属，决定胜负计数
/// 2. [`Role::is_god`]：神职与否，屠边规则按神职/平民分边计数
/// 3. 夜晚行动顺序见 `phase.rs` 的 `NightStage`（守卫 → 狼人 → 女巫 → 预言家）
/// 4. 死亡特效（猎人开枪 / 白痴翻牌）与白天特效（骑士决斗）按角色分支，
///    见 `engine.rs` 的结算逻辑
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// 狼人：夜晚与狼队共同刀一人
    Werewolf,
    /// 预言家：夜晚验一人阵营
    Seer,
    /// 女巫：一瓶解药一瓶毒药，同晚至多用一瓶；解药救当晚刀口
    Witch,
    /// 守卫：夜晚守护一人，不能连续两晚守同一人
    Guard,
    /// 猎人：被刀或被放逐出局时可开枪带走一人；被毒不能开枪
    Hunter,
    /// 白痴：被放逐时翻牌免死但失去投票权；夜晚死亡不触发翻牌
    Idiot,
    /// 骑士：白天可翻牌决斗一人——对方是狼人则狼死，否则骑士死；一局一次
    Knight,
    /// 平民：无技能
    Villager,
}

impl Role {
    /// 阵营归属
    pub fn camp(self) -> Camp {
        match self {
            Role::Werewolf => Camp::Werewolf,
            _ => Camp::Good,
        }
    }

    /// 是否神职（屠边规则里与平民分边计数；白痴翻牌后仍是神职）
    pub fn is_god(self) -> bool {
        !matches!(self, Role::Werewolf | Role::Villager)
    }

    /// 中文展示名（日志与前端直接用）
    pub fn display_name(self) -> &'static str {
        match self {
            Role::Werewolf => "狼人",
            Role::Seer => "预言家",
            Role::Witch => "女巫",
            Role::Guard => "守卫",
            Role::Hunter => "猎人",
            Role::Idiot => "白痴",
            Role::Knight => "骑士",
            Role::Villager => "平民",
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
