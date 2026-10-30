//! 狼人杀领域层 —— 纯类型与规则，零 IO，不依赖 ECS。
//!
//! 模块与原 `werewolf` crate 一致（action/board/error/phase/role/script），
//! 路径经 `werewolf` 门面 re-export 保持外部用法不变。
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod action;
pub mod board;
pub mod error;
pub mod phase;
pub mod rng;
pub mod role;
pub mod script;

pub use action::Action;
pub use board::BoardConfig;
pub use error::{BoardError, GameError};
pub use phase::{
    CheckRecord, CheckResult, DeathCause, NightStage, Outcome, Phase, PlayerId, PlayerRole, Viewer,
    WerewolfReason,
};
pub use rng::Rng;
pub use role::{Camp, Role, RoleProfile};
pub use script::{SCRIPTS, ScriptDef};
