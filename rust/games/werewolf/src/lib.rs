//! `game` —— 狼人杀游戏引擎：纯逻辑状态机，零 IO。
//!
//! ## 定位
//!
//! 对局推进由调用方驱动（AI Agent / server 端点 / 测试）：
//! 读取 [`GameEngine::pending`] 了解当前窗口，喂入 [`Action`]，
//! 引擎校验后推进状态机并返回增量 [`GameEvent`]；
//! 对外视图用 [`GameSnapshot::render`] 按视角裁剪（身份保密、日志过滤）。
//!
//! ## 角色区分（全角色版）
//!
//! | 角色 | 阵营 | 夜晚 | 特效 |
//! | --- | --- | --- | --- |
//! | 狼人 | 狼 | 共同刀人 | — |
//! | 预言家 | 神 | 验一人阵营 | 私有验人记录 |
//! | 女巫 | 神 | 解药救刀口 / 毒药杀人 | 各一瓶，同晚至多一瓶 |
//! | 守卫 | 神 | 守护一人 | 不能连守同一人 |
//! | 猎人 | — | — | 被刀/被放逐可开枪；被毒不能 |
//! | 白痴 | 神 | — | 被放逐翻牌免死，失去投票权 |
//! | 骑士 | 神 | — | 白天决斗一次，验狼成功杀狼否则自尽 |
//! | 平民 | 民 | — | — |
//!
//! 规则要点：夜晚顺序 **守卫 → 狼队夜聊 → 狼刀 → 女巫 → 预言家**
//! （狼队耳语按座位序每狼一条、可沉默，说完自动进刀人窗口；狼聊仅狼队与上帝可见）；
//! 「奶穿」——同一人既被守又被救 ⇒ 死亡；胜负用 **屠边规则**
//! （狼 ≥ 存活神职，或狼 ≥ 存活平民 ⇒ 狼胜；狼全灭 ⇒ 好人胜）。
//!
//! ## 最小用法
//!
//! ```
//! use werewolf::{Action, BoardConfig, GameEngine, GameSnapshot, Viewer};
//!
//! // 9 人标准局：3 狼 + 预女猎 + 3 民（seed 固定 ⇒ 发牌可复现）
//! let mut engine = GameEngine::new(BoardConfig::nine_standard(), 42).unwrap();
//!
//! // 读取当前窗口：夜里等守卫（板子无守卫/角色出局则自动跳段；
//! // 9 人标准局无守卫 ⇒ 第一窗口是狼队夜聊，按座位序每狼一条耳语）
//! let (actor, kind) = engine.pending().unwrap();
//! assert_eq!(kind, "狼人夜聊");
//!
//! // 喂动作推进状态机（狼聊每狼一条后进刀人窗口；完整流程见 tests.rs）
//! let _events = engine
//!     .apply(actor, Action::WolfChat { text: None })
//!     .unwrap_or_default(); // 非当前狼窗口时返回 Err，示例从简
//!
//! // 上帝视角快照（身份全可见；玩家视角只看自己/狼队友/翻牌者）
//! let snapshot = GameSnapshot::render(&engine, Viewer::God).unwrap();
//! assert!(!snapshot.logs.is_empty());
//! ```
//!
//! 服务端接线（房间管理 / SSE 快照推送）与 LLM 决策层后续分层接入；
//! 对齐的前端类型见 `apps/web/src/werewolf/types.ts`（`GameSnapshot` 观战页契约）。
//!
//! ## 工具暴露
//!
//! [`tools`] 模块把每个行动窗口包成可被 LLM 调用的工具
//! （基类契约来自 workspace 的 `tools` 包）：狼刀 / 女巫用药 / 预言家验人 /
//! 发言 / 投票等，用 [`tools::registry`] 一次拿齐。

pub mod action;
pub mod board;
pub mod engine;
pub mod error;
pub mod event;
pub mod phase;
pub mod player;
pub mod role;
pub mod snapshot;
pub mod summary;
pub mod tools;

#[cfg(test)]
mod tests;

pub use action::Action;
pub use board::{BoardConfig, BoardError};
pub use engine::GameEngine;
pub use error::GameError;
pub use event::{GameEvent, NightDeath};
pub use phase::{
    CheckRecord, CheckResult, DeathCause, NightStage, Outcome, Phase, PlayerId, PlayerRole, Viewer,
    WerewolfReason,
};
pub use player::{PlayerState, SnapshotPlayer};
pub use role::{Camp, Role, RoleProfile};
pub use snapshot::{GameSnapshot, PendingWindow, SnapshotLog};
pub use summary::{GameSummary, RoleStatus};
