//! 玩家内部状态（引擎视图行类型，纯数据无逻辑）。

use serde::{Deserialize, Serialize};

use crate::phase::{CheckRecord, PlayerId, PlayerRole};
use crate::role::Role;

/// 玩家内部状态（引擎私有，不直接序列化——身份保密靠快照层的视角裁剪）
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

/// 序列化用空壳：PlayerState 不直接落盘（保密视图在快照层）
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
struct PlayerStateSerdeMarker;

// 让 PlayerState 保持"纯数据"定位：serde 只给需要落库的快照类型（SnapshotPlayer）
#[allow(unused_imports)]
use PlayerStateSerdeMarker as _;
