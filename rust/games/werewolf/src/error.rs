/// 对局规则错误：调用方喂入了阶段不符、越权或非法目标的动作。
///
/// 这是领域规则错误（非系统错误）；server 接线时再决定如何映射到 HTTP 状态码
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GameError {
    /// 对局已结束，不再接收动作
    #[error("对局已结束")]
    GameOver,
    /// 动作与当前阶段不符（如白天喂了夜晚动作）
    #[error("当前阶段不接受该动作：{0}")]
    WrongPhase(&'static str),
    /// 动作不是当前窗口角色发起的（座位越权）
    #[error("当前轮到 {expected} 号行动，收到 {actual} 号的动作")]
    NotYourTurn { expected: u8, actual: u8 },
    /// 目标座位不存在
    #[error("座位 {seat} 不存在（有效范围 0..{max}）")]
    NoSuchPlayer { seat: u8, max: u8 },
    /// 目标已出局（投票/刀人/守护/验人的目标必须是活人）
    #[error("目标 {0} 已出局")]
    TargetDead(u8),
    /// 不能以自己为目标（预言家验人 / 女巫毒人）
    #[error("{0}不能以自己为目标")]
    SelfTarget(&'static str),
    /// 守卫连续两晚守同一人
    #[error("守卫不能连续两晚守护同一人")]
    GuardRepeat,
    /// 女巫同晚同时用解药和毒药（经典规则：同晚至多用一瓶）
    #[error("女巫同晚至多用一瓶药")]
    WitchBothPotions,
    /// 药已用完（解药/毒药各一瓶）
    #[error("女巫的{0}已用完")]
    PotionDepleted(&'static str),
    /// 女巫想救人但今晚没有刀口（空刀夜不能空救）
    #[error("今晚没有刀口，无法使用解药")]
    NoKillToSave,
    /// 骑士一局只能决斗一次
    #[error("骑士一局只能决斗一次")]
    DuelUsed,
    /// 已投过票（本轮投票权已用）
    #[error("{0} 号本轮已投票")]
    AlreadyVoted(u8),
    /// 发言顺序不对：动作来自非当前发言者
    #[error("发言顺序不对：当前该 {expected} 号发言，收到 {actual} 号")]
    NotSpeakerTurn { expected: u8, actual: u8 },
    /// 发言文本为空
    #[error("发言不能为空")]
    EmptySpeech,
}

/// 非法动作的便捷构造
impl GameError {
    /// 座位越界检查
    pub fn check_seat(seat: u8, total: u8) -> Result<(), Self> {
        if seat >= total {
            return Err(Self::NoSuchPlayer { seat, max: total });
        }
        Ok(())
    }
}

/// 板子配置错误：人数约束违反
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BoardError {
    #[error("板子至少需要 1 个狼人")]
    NoWerewolf,
    #[error("板子至少需要 1 个好人角色")]
    NoGood,
    #[error("神职 {0} 至多 1 枚（引擎按单枚神职实现夜晚窗口）")]
    DuplicateGod(String),
}
