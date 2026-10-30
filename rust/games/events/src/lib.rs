//! 狼人杀事件层 —— [`GameEvent`] 业务事实流（seq 单调、可回放）。
//!
//! 事件日志是唯一事实源：memory / mvp / SSE / 前端回放全部从它派生。
//! 与 ECS 内部 Message 严格区分；不依赖 ECS。
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod event;

pub use event::{GameEvent, NightDeath};
