//! 角色档案：发给 AI 扮演角色时的说明素材。
//!
//! 每个角色一个文件（`werewolf.rs` / `seer.rs` / ...），本模块只定义档案结构与查表。
//! 档案是**纯描述数据**（上帝视角，含未翻牌信息），供 AI 扮演与前端展示共用；
//! 引擎的行为规则仍以 `engine.rs` 的结算逻辑为准，档案描述必须与其一致。

use serde::Serialize;

use super::{Camp, Role};
use super::{guard, hunter, idiot, knight, seer, villager, werewolf, witch};

/// 单个角色的完整档案
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoleProfile {
    /// 角色枚举
    pub role: Role,
    /// 阵营
    pub camp: Camp,
    /// 中文展示名
    pub name: &'static str,
    /// 展示图标（前端用）
    pub emoji: &'static str,
    /// 角色描述（你是谁）
    pub description: &'static str,
    /// 能做什么（逐条）
    pub abilities: &'static [&'static str],
    /// 规则约束（不能做什么）
    pub constraints: &'static [&'static str],
    /// 夜晚行动说明（无夜晚技能为 `None`）
    pub night_action: Option<&'static str>,
    /// 胜利条件
    pub win_condition: &'static str,
}

impl RoleProfile {
    /// 渲染成发给 AI 的角色说明文本（system prompt 素材）
    pub fn prompt_text(&self) -> String {
        let mut lines = vec![format!(
            "【{}】{}（{}）",
            self.name,
            self.emoji,
            camp_name(self.camp)
        )];
        lines.push(self.description.to_string());
        lines.push("你能做什么：".into());
        lines.extend(self.abilities.iter().map(|a| format!("- {a}")));
        if !self.constraints.is_empty() {
            lines.push("规则约束：".into());
            lines.extend(self.constraints.iter().map(|c| format!("- {c}")));
        }
        if let Some(night) = self.night_action {
            lines.push(format!("夜晚行动：{night}"));
        }
        lines.push(format!("胜利条件：{}", self.win_condition));
        lines.join("\n")
    }
}

fn camp_name(camp: Camp) -> &'static str {
    match camp {
        Camp::Werewolf => "狼人阵营",
        Camp::Good => "好人阵营",
    }
}

/// 角色档案查表
pub fn profile(role: Role) -> &'static RoleProfile {
    match role {
        Role::Werewolf => &werewolf::WEREWOLF,
        Role::Seer => &seer::SEER,
        Role::Witch => &witch::WITCH,
        Role::Guard => &guard::GUARD,
        Role::Hunter => &hunter::HUNTER,
        Role::Idiot => &idiot::IDIOT,
        Role::Knight => &knight::KNIGHT,
        Role::Villager => &villager::VILLAGER,
    }
}

/// 全部角色档案（按 [`Role`] 枚举序）
pub fn all_profiles() -> Vec<&'static RoleProfile> {
    [
        Role::Werewolf,
        Role::Seer,
        Role::Witch,
        Role::Guard,
        Role::Hunter,
        Role::Idiot,
        Role::Knight,
        Role::Villager,
    ]
    .into_iter()
    .map(profile)
    .collect()
}
