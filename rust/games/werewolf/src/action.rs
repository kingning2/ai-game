use serde::{Deserialize, Serialize};

use crate::phase::PlayerId;

/// 调用方喂入引擎的动作。合法性由引擎校验：
/// 阶段不符 / 不是该角色窗口 / 目标越界或已死 → [`GameError`](crate::error::GameError)。
///
/// 夜晚动作里"放弃"用 `None` 表达（守卫可以空守、女巫可以不用药、
/// 猎人可以不开枪、骑士可以不决斗），白天发言与投票有独立的"跳过"语义
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Action {
    /// 守卫守护（`None` = 今晚不守）；不能与昨晚守同一人
    GuardProtect { target: Option<PlayerId> },
    /// 狼队夜聊耳语（`None` = 本轮保持沉默）；
    /// 按座位序每狼一条，说完自动进刀人窗口
    WolfChat { text: Option<String> },
    /// 狼队共同刀一人（多狼共享一次决定）
    WerewolfKill { target: PlayerId },
    /// 女巫行动：是否用解药（救当晚刀口）+ 是否毒人；同晚至多用一瓶
    WitchAct {
        save: bool,
        poison: Option<PlayerId>,
    },
    /// 预言家验人（不能验自己）
    SeerCheck { target: PlayerId },
    /// 当前发言者发言（发言者由引擎按座位序轮转，调用方只喂文本）
    Speech { text: String },
    /// 投票（`None` = 弃票；目标须为活人）
    Vote { target: Option<PlayerId> },
    /// 猎人开枪（`None` = 放弃开枪）
    HunterShot { target: Option<PlayerId> },
    /// 骑士决斗（`None` = 放弃决斗，进入投票）
    KnightDuel { target: Option<PlayerId> },
}

impl Action {
    /// 上帝日志用的一句话描述
    pub fn describe(&self) -> String {
        match self {
            Action::GuardProtect { target } => match target {
                Some(t) => format!("守卫守护了 {t} 号"),
                None => "守卫今晚空守".to_string(),
            },
            Action::WolfChat { text } => match text {
                Some(t) => format!("狼队耳语：「{t}」"),
                None => "狼队耳语中保持沉默".to_string(),
            },
            Action::WerewolfKill { target } => format!("狼队袭击了 {target} 号"),
            Action::WitchAct { save, poison } => match (save, poison) {
                (true, Some(p)) => format!("女巫救下刀口并毒死了 {p} 号"),
                (true, None) => "女巫用解药救下刀口".to_string(),
                (false, Some(p)) => format!("女巫毒死了 {p} 号"),
                (false, None) => "女巫按兵不动".to_string(),
            },
            Action::SeerCheck { target } => format!("预言家查验了 {target} 号"),
            Action::Speech { text } => format!("发言：{text}"),
            Action::Vote { target } => match target {
                Some(t) => format!("投给 {t} 号"),
                None => "弃票".to_string(),
            },
            Action::HunterShot { target } => match target {
                Some(t) => format!("猎人开枪带走了 {t} 号"),
                None => "猎人放弃开枪".to_string(),
            },
            Action::KnightDuel { target } => match target {
                Some(t) => format!("骑士发起决斗，对象 {t} 号"),
                None => "骑士放弃决斗".to_string(),
            },
        }
    }
}
