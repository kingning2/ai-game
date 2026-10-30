//! 剧本定义 —— 「一个剧本 = 一种玩法」的单一事实源。
//!
//! 剧本 = 板子（[`BoardConfig`] 预设）+ 规则文案 + 主持开场词，
//! 服务端 `GET /api/scripts` 下发给前端做玩法选择卡与介绍页；
//! 开局仍按剧本 id（= 板子预设名）走 `POST /api/game/start`，引擎照常校验。
//! 新增玩法：在 [`SCRIPTS`] 加一条 + [`ScriptDef::board`] 加对应预设分支 + 两侧测试。

use serde::Serialize;

use crate::board::BoardConfig;

/// 剧本：玩法选择卡与介绍页的全部素材（板子由 id 映射到引擎预设）
#[derive(Debug, Clone, Serialize)]
pub struct ScriptDef {
    /// 剧本 id（= 板子预设名，`GameState::resolve_board` 可解析）
    pub id: &'static str,
    pub name: &'static str,
    /// 玩家座位数（= 板子总人数）
    pub total: u8,
    /// 一句话简介（选择卡副标题）
    pub desc: &'static str,
    /// 角色配比（(角色代码, 数量)，选择卡 chips 与介绍页板子卡共用）
    pub roles: &'static [(&'static str, u8)],
    /// 规则列表（介绍页逐条展示；屠边/夜晚顺序/奶穿等本剧本实际生效的规则）
    pub rules: &'static [&'static str],
    /// 主持开场词（介绍页主持人逐段播报）
    pub intro: &'static [&'static str],
}

/// 内置剧本（前端玩法选择卡按此渲染；顺序即展示顺序）
pub const SCRIPTS: &[ScriptDef] = &[
    ScriptDef {
        id: "newbie_six",
        name: "新手 6 人局",
        total: 6,
        desc: "2 狼 · 预言家 · 女巫 · 2 民，先学会听发言",
        roles: &[("werewolf", 2), ("seer", 1), ("witch", 1), ("villager", 2)],
        rules: &[
            "夜晚顺序：狼刀 → 女巫（解药/毒药各一瓶，同晚至多一瓶）→ 预言家查验",
            "天亮公布死讯（或平安夜），依次发言后全员投票放逐一人",
            "屠边规则：狼人数 ≥ 存活神职，或 ≥ 存活平民，狼人获胜",
            "女巫解药交出后，天亮死讯照常公布",
        ],
        intro: &[
            "欢迎来到新手 6 人局。六位选手，两匹狼，一位预言家，一位女巫，两位平民。",
            "狼人每晚共同刀一人；预言家每晚查验一人阵营；女巫手握一瓶解药一瓶毒药。",
            "白天所有人依次发言，投票放逐怀疑对象。好人找出所有狼即胜；狼人杀到屠边即胜。",
        ],
    },
    ScriptDef {
        id: "nine_standard",
        name: "标准 9 人局",
        total: 9,
        desc: "3 狼 · 预女猎 · 3 民，屠边规则的经典局",
        roles: &[
            ("werewolf", 3),
            ("seer", 1),
            ("witch", 1),
            ("hunter", 1),
            ("villager", 3),
        ],
        rules: &[
            "夜晚顺序：狼队夜聊 → 狼刀 → 女巫 → 预言家查验",
            "女巫解药/毒药各一瓶，同晚至多使用一瓶；解药救下刀口则平安夜",
            "猎人被刀或被放逐时可开枪带走一人；被毒杀则无法开枪",
            "天亮公布死讯，出局者留遗言，依次发言后投票放逐",
            "屠边规则：狼人数 ≥ 存活神职，或 ≥ 存活平民，狼人获胜",
        ],
        intro: &[
            "欢迎来到标准 9 人局。三匹狼潜伏其中，预言家、女巫、猎人三位神职守卫好人阵营。",
            "狼队夜里耳语商议刀口；女巫权衡解药与毒药；猎人在倒下时扣动扳机。",
            "白天发言与投票是唯一的武器。好人放逐所有狼人即胜；狼人屠边即胜。",
        ],
    },
    ScriptDef {
        id: "all_stars",
        name: "全明星 12 人局",
        total: 12,
        desc: "4 狼 · 预女猎守 · 4 民，守卫加入的进阶局",
        roles: &[
            ("werewolf", 4),
            ("seer", 1),
            ("witch", 1),
            ("guard", 1),
            ("hunter", 1),
            ("villager", 4),
        ],
        rules: &[
            "夜晚顺序：守卫守护 → 狼队夜聊 → 狼刀 → 女巫 → 预言家查验",
            "守卫每晚守护一人挡下狼刀，但不能连续两晚守护同一人",
            "「奶穿」：同一人既被守卫守护又被女巫解救，仍然死亡",
            "女巫解药/毒药各一瓶，同晚至多一瓶；猎人被刀或被放逐可开枪，被毒不能",
            "屠边规则：狼人数 ≥ 存活神职，或 ≥ 存活平民，狼人获胜",
        ],
        intro: &[
            "欢迎来到全明星 12 人局。四匹狼对上四位神职，守卫的盾牌首次登场。",
            "守卫每晚守护一人，但不能连守同一人；守卫与女巫同时救下刀口，就是致命的「奶穿」。",
            "十二位选手，四天四夜的言语博弈。好人放逐所有狼人即胜；狼人屠边即胜。",
        ],
    },
];

/// 按 id 取剧本
#[must_use]
pub fn script(id: &str) -> Option<&'static ScriptDef> {
    SCRIPTS.iter().find(|s| s.id == id)
}

#[cfg(test)]
#[path = "script_tests.rs"]
mod tests;

impl ScriptDef {
    /// 剧本 → 引擎板子（发牌与校验仍由 [`BoardConfig`] 承担；
    /// id 与 `GameState::resolve_board` 的预设名一一对应，服务端测试锁定一致性）
    #[must_use]
    pub fn board(&self) -> BoardConfig {
        match self.id {
            "newbie_six" => BoardConfig::newbie_six(),
            "all_stars" => BoardConfig::all_stars(),
            _ => BoardConfig::nine_standard(),
        }
    }
}
