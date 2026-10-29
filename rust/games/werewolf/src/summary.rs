//! 单局全局总览：剩余人数与身份分布。
//!
//! 从引擎玩家状态派生的纯查询视图（零 IO）。**上帝/主持视角**——
//! 含未翻牌身份，禁止直接下发给玩家；玩家侧总览走 [`crate::snapshot`] 的视角裁剪。

use serde::{Deserialize, Serialize};

use crate::engine::GameEngine;
use crate::role::Role;

/// 单个身份的存活统计
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleStatus {
    /// 身份
    pub role: Role,
    /// 该身份总人数
    pub total: u8,
    /// 仍然存活的人数
    pub alive: u8,
}

/// 单局全局总览
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSummary {
    /// 当前天数（第一夜为 1；结算后为最后活跃天数）
    pub day: u32,
    /// 全部座位数
    pub total: u8,
    /// 存活人数
    pub alive_total: u8,
    /// 各身份剩余情况（按 [`Role`] 枚举序；板子里没有的身份不出现）
    pub roles: Vec<RoleStatus>,
}

impl GameSummary {
    /// 从引擎当前状态派生总览
    pub fn of(engine: &GameEngine) -> Self {
        let players = engine.players();
        let mut roles: Vec<RoleStatus> = Vec::new();
        for player in players {
            match roles.iter_mut().find(|entry| entry.role == player.role) {
                Some(entry) => {
                    entry.total += 1;
                    if player.alive {
                        entry.alive += 1;
                    }
                }
                None => roles.push(RoleStatus {
                    role: player.role,
                    total: 1,
                    alive: u8::from(player.alive),
                }),
            }
        }
        roles.sort_by_key(|entry| entry.role);

        Self {
            day: engine.phase().day(),
            total: engine.total_players(),
            alive_total: players.iter().filter(|p| p.alive).count() as u8,
            roles,
        }
    }
}
