use serde::{Deserialize, Serialize};

pub mod guard;
pub mod hunter;
pub mod idiot;
pub mod knight;
pub mod profile;
pub mod seer;
pub mod villager;
pub mod werewolf;
pub mod witch;

pub use profile::{RoleProfile, all_profiles, profile};

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
/// 3. [`Role::night_order`]：夜晚行动顺序（守卫 → 狼人 → 女巫 → 预言家）
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

    /// 夜晚行动顺序（小的先行动）；无夜晚技能的角色返回 `None`。
    /// 经典顺序：守卫(1) → 狼人(2) → 女巫(3) → 预言家(4)。
    /// 守卫先行动，狼队才知道守了谁（刀人不受守卫影响，由结算判定平安夜）
    pub fn night_order(self) -> Option<u8> {
        match self {
            Role::Guard => Some(1),
            Role::Werewolf => Some(2),
            Role::Witch => Some(3),
            Role::Seer => Some(4),
            _ => None,
        }
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
mod tests {
    use super::*;

    #[test]
    fn camps_and_gods_are_consistent() {
        assert_eq!(Role::Werewolf.camp(), Camp::Werewolf);
        for role in [
            Role::Seer,
            Role::Witch,
            Role::Guard,
            Role::Hunter,
            Role::Idiot,
            Role::Knight,
            Role::Villager,
        ] {
            assert_eq!(role.camp(), Camp::Good, "{role:?} 应属好人阵营");
            assert_eq!(
                role.is_god(),
                role != Role::Villager,
                "{role:?} 神职判定错误"
            );
        }
    }

    #[test]
    fn night_order_is_fixed() {
        let mut ordered: Vec<(u8, Role)> = [Role::Guard, Role::Werewolf, Role::Witch, Role::Seer]
            .into_iter()
            .map(|r| (r.night_order().expect("四个角色都有夜序"), r))
            .collect();
        ordered.sort_by_key(|(order, _)| *order);
        assert_eq!(
            ordered.into_iter().map(|(_, r)| r).collect::<Vec<_>>(),
            vec![Role::Guard, Role::Werewolf, Role::Witch, Role::Seer],
        );
        assert!(Role::Villager.night_order().is_none());
    }

    #[test]
    fn serde_uses_lowercase() {
        assert_eq!(
            serde_json::to_string(&Role::Werewolf).unwrap(),
            "\"werewolf\""
        );
        assert_eq!(
            serde_json::from_str::<Role>("\"seer\"").unwrap(),
            Role::Seer
        );
    }
}
