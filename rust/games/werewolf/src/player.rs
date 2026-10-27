use serde::{Deserialize, Serialize};

use crate::phase::{CheckRecord, PlayerId, PlayerRole};
use crate::role::Role;

/// 玩家内部状态（引擎私有，不直接序列化——身份保密靠 `snapshot.rs` 的视角裁剪）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerState {
    pub id: PlayerId,
    pub role: Role,
    pub alive: bool,
    /// 白痴翻牌后置 false（失去投票权）
    pub can_vote: bool,
    /// 白痴已翻牌（放逐免死只触发一次）
    pub revealed_idiot: bool,
    /// 骑士已用过决斗权
    pub duel_used: bool,
    /// 预言家的验人记录（挂玩家身上，省一个旁表）
    pub checks: Vec<CheckRecord>,
}

impl PlayerState {
    pub fn new(id: PlayerId, role: Role) -> Self {
        Self {
            id,
            role,
            alive: true,
            can_vote: true,
            revealed_idiot: false,
            duel_used: false,
            checks: Vec::new(),
        }
    }

    /// 上帝视角点名用
    pub fn named(&self) -> PlayerRole {
        PlayerRole {
            id: self.id,
            role: self.role,
        }
    }

    /// 预言家的验人记录（挂玩家身上，省一个旁表）
    pub fn checks(&self) -> &[CheckRecord] {
        &self.checks
    }
}

/// 快照里的玩家（对外可见的安全形态：`role` 只有上帝视角与本人可见，
/// 玩家视角看别人全是 `None`；狼人视角看狼队友是 `Some`）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotPlayer {
    pub id: PlayerId,
    pub alive: bool,
    /// 已公开的身份（死者翻牌 / 白痴翻牌）；活人一律 `None`（对玩家视角）
    pub role: Option<Role>,
    /// 上帝视角才为 true
    pub can_vote: bool,
}
