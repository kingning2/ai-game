//! 角色档案：发给 AI 扮演角色时的说明素材。
//!
//! 档案是**纯描述数据**（上帝视角，含未翻牌信息），供 AI 扮演与前端展示共用；
//! 引擎的行为规则仍以 `engine.rs` 的结算逻辑为准，档案描述必须与其一致。

use serde::Serialize;

use super::Role;

/// 单个角色的完整档案
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoleProfile {
    /// 角色枚举
    pub role: Role,
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

/// 全部角色档案，按 [`Role`] 枚举序排布，[`profile`] 按下标查表
static PROFILES: [RoleProfile; 8] = [
    // 狼人
    RoleProfile {
        role: Role::Werewolf,
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
    },
    // 预言家
    RoleProfile {
        role: Role::Seer,
        emoji: "🔮",
        description: "你是预言家，好人阵营的核心神职。每晚查验一名玩家的阵营，用验人结果带领好人找出狼人。",
        abilities: &[
            "每晚查验一名玩家的阵营（狼人 / 好人）",
            "全部验人记录仅自己可见，其他玩家（包括上帝视角快照）无法查看",
        ],
        constraints: &["每晚至多查验一人", "不能查验自己", "查验目标必须是活人"],
        night_action: Some("选择一名玩家查验其阵营"),
        win_condition: "放逐所有狼人（好人阵营胜利）",
    },
    // 女巫
    RoleProfile {
        role: Role::Witch,
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
    },
    // 守卫
    RoleProfile {
        role: Role::Guard,
        emoji: "🛡️",
        description: "你是守卫，好人阵营的神职。每晚守护一名玩家抵挡狼人刀口，是好人阵营的防御核心。",
        abilities: &["每晚守护一名玩家：被守护者当晚不会被狼刀杀死"],
        constraints: &[
            "不能连续两晚守护同一个人",
            "守护目标必须是活人",
            "同晚守卫与女巫解药作用于同一人时该玩家仍会死亡（奶穿规则）",
        ],
        night_action: Some("选择今晚守护的玩家（也可选择不守）"),
        win_condition: "放逐所有狼人（好人阵营胜利）",
    },
    // 猎人
    RoleProfile {
        role: Role::Hunter,
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
    },
    // 白痴
    RoleProfile {
        role: Role::Idiot,
        emoji: "🤡",
        description: "你是白痴，好人阵营的特殊神职。被投票放逐时翻牌亮明身份免死，但从此失去投票权。",
        abilities: &["被投票放逐时翻牌亮明身份，免于出局，继续留在场上"],
        constraints: &[
            "翻牌免死后失去投票权，只能发言不能再投票",
            "夜晚被刀杀或被毒死不触发翻牌，直接出局",
        ],
        night_action: None,
        win_condition: "放逐所有狼人（好人阵营胜利）",
    },
    // 骑士
    RoleProfile {
        role: Role::Knight,
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
    },
    // 平民
    RoleProfile {
        role: Role::Villager,
        emoji: "👤",
        description: "你是平民，没有任何主动技能。你的武器只有发言、倾听与投票——用逻辑找出狼人。",
        abilities: &["白天发言与投票放逐嫌疑人"],
        constraints: &["没有夜晚技能，夜晚只能等待天亮"],
        night_action: None,
        win_condition: "放逐所有狼人（好人阵营胜利）",
    },
];

/// 角色档案查表
pub fn profile(role: Role) -> &'static RoleProfile {
    &PROFILES[role as usize]
}
