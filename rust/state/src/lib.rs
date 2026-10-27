#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! 应用配置与全局状态管理。

mod agent_state;
mod app_state;
mod config;
mod error;
mod game_state;
mod roster_state;
mod stats;
mod track;

pub use agent_state::AgentState;
pub use app_state::AppState;
pub use config::AppConfig;
pub use error::StateError;
pub use game_state::{GameState, GameStatus, GameView, InvalidTransition};
pub use roster_state::{PlayerOverride, ROSTER_PLAYERS, RosterState, SettingsPatch};
pub use stats::Stats;
pub use track::track;
