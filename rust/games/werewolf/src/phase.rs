use serde::{Deserialize, Serialize};

pub use crate::role::Role;

/// 对局阶段。状态机流向：
/// `Night → DaySpeech → (KnightDuel?) → DayVote → (HunterShot?)* → Night …`
/// 直到 [`Phase::Settled`]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Phase {
    /// 夜晚：按 [`NightStage`] 顺序等待各角色行动
    Night {
        /// 天数（第一夜为 1）
        day: u32,
        stage: NightStage,
    },
    /// 白天发言：按座位序从死者下一位开始轮流发言
    DaySpeech {
        day: u32,
        /// 下一个该发言的座位
        next_speaker: u8,
    },
    /// 白天决斗窗口：骑士存活且未用过决斗权时出现（可跳过）
    KnightDuel { day: u32, knight: u8 },
    /// 白天投票放逐
    DayVote { day: u32 },
    /// 猎人开枪窗口（可跳过）；级联：枪死者若也是猎人则接续新窗口
    HunterShot { day: u32, hunter: u8 },
    /// 胜负已分，对局结束
    Settled { outcome: Outcome },
}

impl Phase {
    /// 当前天数；结算后返回最后一夜/一天的天数（仅供日志）
    pub fn day(&self) -> u32 {
        match self {
            Phase::Night { day, .. }
            | Phase::DaySpeech { day, .. }
            | Phase::KnightDuel { day, .. }
            | Phase::DayVote { day }
            | Phase::HunterShot { day, .. } => *day,
            Phase::Settled { .. } => 0,
        }
    }
}

/// 夜晚行动顺序（与 [`Role::night_order`](crate::role::Role::night_order) 对齐）。
/// 狼队先在 [`NightStage::WerewolfChat`] 按座位序耳语商量，
/// 再共享一次刀（[`NightStage::Werewolves`]，引擎只收一个决定）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NightStage {
    Guard,
    /// 狼队夜聊：每狼一条耳语（可沉默），说完自动进刀人窗口
    WerewolfChat,
    Werewolves,
    Witch,
    Seer,
}

impl NightStage {
    /// 该窗口对应的角色
    pub fn role(self) -> Role {
        match self {
            NightStage::Guard => Role::Guard,
            NightStage::WerewolfChat | NightStage::Werewolves => Role::Werewolf,
            NightStage::Witch => Role::Witch,
            NightStage::Seer => Role::Seer,
        }
    }

    /// 下一段（最后一段返回 `None`，触发结算）
    pub fn next(self) -> Option<Self> {
        match self {
            NightStage::Guard => Some(NightStage::WerewolfChat),
            NightStage::WerewolfChat => Some(NightStage::Werewolves),
            NightStage::Werewolves => Some(NightStage::Witch),
            NightStage::Witch => Some(NightStage::Seer),
            NightStage::Seer => None,
        }
    }
}

/// 胜负结果（屠边规则）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "winner", rename_all = "lowercase")]
pub enum Outcome {
    /// 狼人胜：杀光神职（屠神）或杀光平民（屠民）
    #[serde(rename = "werewolf", rename_all = "camelCase")]
    WerewolfWin { reason: WerewolfReason },
    /// 好人胜：狼人全灭
    #[serde(rename = "good")]
    GoodWin,
}

/// 狼人胜利原因
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WerewolfReason {
    /// 屠神：存活狼人数 ≥ 存活神职数
    GodsWiped,
    /// 屠民：存活狼人数 ≥ 存活平民数
    VillagersWiped,
}

/// 死亡原因（上帝视角事件用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeathCause {
    /// 夜晚被刀
    Slain,
    /// 被女巫毒死（被毒的猎人不能开枪）
    Poisoned,
    /// 白天投票放逐
    Exiled,
    /// 被猎人开枪带走
    Shot,
    /// 骑士决斗失败被杀
    Duel,
}

/// 预言家验人结果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckResult {
    Good,
    Werewolf,
}

/// 预言家的私有验人记录（快照里只对预言家本人可见）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckRecord {
    pub day: u32,
    pub target: u8,
    pub result: CheckResult,
}

/// 玩家座位号（0 开始，发牌时固定）
pub type PlayerId = u8;

/// 玩家身份归属（视角选择）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Viewer {
    /// 上帝视角：看全部身份与所有事件（观战页）
    God,
    /// 玩家视角：只看自己的身份、夜晚私有结果、狼队友
    Player(PlayerId),
}

/// 玩家侧角色提示（日志/事件里点名时区分阵营身份，供上帝日志使用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct PlayerRole {
    pub id: PlayerId,
    pub role: Role,
}
