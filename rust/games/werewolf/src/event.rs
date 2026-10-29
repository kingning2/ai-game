use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::phase::{DeathCause, Outcome, PlayerId, PlayerRole};

/// 引擎产出的对局事件——既是对局日志（前端 EventFeed 数据源），
/// 也是调用方推进 LLM Agent 决策的事实流。
///
/// **上帝视角事件**（夜晚动作细节）只能进上帝日志；玩家视角的日志裁剪在
/// `snapshot.rs` 完成，引擎事件流本身保持完整（供回放/审计）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GameEvent {
    /// 开局：板子校验通过、发牌完成
    GameStarted {
        players: u8,
        /// 各角色数量（上帝视角）
        board: Vec<(String, u8)>,
    },
    /// 天黑
    NightBegan { day: u32 },
    /// 某角色的夜晚动作已收（上帝视角）
    NightAction {
        day: u32,
        actor: PlayerRole,
        action: Action,
    },
    /// 狼队夜聊耳语（仅上帝与狼队可见；调用方决策层据此同步狼队意图）
    WolfChatSpoke {
        day: u32,
        speaker: PlayerRole,
        text: String,
    },
    /// 狼队成员夜聊轮空（保持沉默；可见性同 [`GameEvent::WolfChatSpoke`]）
    WolfChatPassed { day: u32, speaker: PlayerRole },
    /// 天亮：公布夜晚死者（平安夜则空列表）
    Dawn { day: u32, deaths: Vec<NightDeath> },
    /// 有人发言
    SpeechMade {
        day: u32,
        speaker: PlayerId,
        text: String,
    },
    /// 骑士决斗结果（放弃决斗不产生事件）
    KnightDuelResolved {
        day: u32,
        knight: PlayerId,
        target: PlayerId,
        target_is_wolf: bool,
        /// 决斗死者（对方是狼则狼死，否则骑士死）
        loser: PlayerRole,
    },
    /// 投票结果（平票时无人放逐）
    VoteResolved {
        day: u32,
        /// 座位号 → 得票（只含得票 > 0 的）
        counts: Vec<(PlayerId, u8)>,
        /// 平票（无人放逐）
        tie: bool,
        exiled: Option<PlayerRole>,
    },
    /// 白痴翻牌免死（失去投票权）
    IdiotRevealed { day: u32, idiot: PlayerId },
    /// 猎人开枪（放弃开枪也产生事件，target 为 None）
    HunterShotFired {
        day: u32,
        hunter: PlayerRole,
        target: Option<PlayerRole>,
    },
    /// 有人出局（含死因翻牌，上帝视角）——夜晚死者/枪死者/决斗死者/放逐者统一在此广播
    PlayerDied {
        day: u32,
        player: PlayerRole,
        cause: DeathCause,
    },
    /// 对局结束
    GameOver { outcome: Outcome },
}

impl GameEvent {
    /// 事件所属天数（开局/终局/夜间暗中行动为 0）
    pub fn day(&self) -> u32 {
        match self {
            GameEvent::GameStarted { .. }
            | GameEvent::GameOver { .. }
            | GameEvent::NightAction { .. } => 0,
            GameEvent::NightBegan { day }
            | GameEvent::Dawn { day, .. }
            | GameEvent::SpeechMade { day, .. }
            | GameEvent::KnightDuelResolved { day, .. }
            | GameEvent::VoteResolved { day, .. }
            | GameEvent::IdiotRevealed { day, .. }
            | GameEvent::HunterShotFired { day, .. }
            | GameEvent::PlayerDied { day, .. }
            | GameEvent::WolfChatSpoke { day, .. }
            | GameEvent::WolfChatPassed { day, .. } => *day,
        }
    }
}

/// 夜晚死者（天亮公布用）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NightDeath {
    pub player: PlayerId,
    pub cause: DeathCause,
}
