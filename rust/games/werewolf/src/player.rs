//! 玩家状态与快照视图类型。
//!
//! `PlayerState` 已下沉 `game-domain`（引擎视图行类型），此处 re-export
//! 保持 `werewolf::player::PlayerState` 路径稳定。

use serde::{Deserialize, Serialize};

use game_domain::phase::PlayerId;
use game_domain::role::Role;

pub use game_domain::player::PlayerState;

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
