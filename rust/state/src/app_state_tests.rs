//! app_state.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

#[test]
fn uptime_starts_at_zero() {
    let state = AppState::new(AppConfig::default(), AgentState::ephemeral());
    assert_eq!(state.uptime_secs(), 0);
    assert_eq!(state.config.port, 17870);
    assert_eq!(state.stats.total(), 0);
}
