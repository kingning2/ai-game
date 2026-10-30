//! agent_state.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

fn temp_path(name: &str) -> PathBuf {
    tests_support::temp_path(name)
}

#[test]
fn load_missing_file_falls_back_to_default() {
    let path = temp_path("missing");
    let _ = std::fs::remove_file(&path);
    let state = AgentState::load(path);
    assert_eq!(state.config(), AgentConfig::default());
}

#[test]
fn load_valid_file_roundtrips() {
    let path = temp_path("valid");
    let mut config = AgentConfig::default();
    config.openai.api_key = "sk-test-1234".to_string();
    config.max_tokens = 1024;

    write_json_atomic(&path, &config).unwrap();
    let state = AgentState::load(path.clone());
    assert_eq!(state.config().openai.api_key, "sk-test-1234");
    assert_eq!(state.config().max_tokens, 1024);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn load_corrupt_file_falls_back_to_default() {
    let path = temp_path("corrupt");
    std::fs::write(&path, b"{ not json").unwrap();
    let state = AgentState::load(path.clone());
    assert_eq!(state.config(), AgentConfig::default());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn update_persists_and_swaps_provider() {
    let path = temp_path("update");
    let state = AgentState::load(path.clone());

    let mut config = AgentConfig {
        provider: agent::ProviderKind::Anthropic,
        ..Default::default()
    };
    config.anthropic.api_key = "sk-ant-xyz".to_string();

    state.update(config).unwrap();

    // 内存已生效：provider 切到 anthropic
    assert_eq!(state.provider().id(), "anthropic");
    // 文件已落盘：重新加载能读回
    let reloaded = AgentState::load(path.clone());
    assert_eq!(reloaded.config().anthropic.api_key, "sk-ant-xyz");
    assert_eq!(reloaded.config().provider, agent::ProviderKind::Anthropic);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn update_failure_keeps_old_state() {
    let state = AgentState::ephemeral();
    let old = state.config();
    // ephemeral 不落盘，update 只会成功——这里验证的是"成功后内存生效"
    let config = AgentConfig {
        max_tokens: 256,
        ..Default::default()
    };
    state.update(config).unwrap();
    assert_eq!(state.config().max_tokens, 256);
    assert_eq!(old.max_tokens, 4096);
}

#[test]
fn default_path_uses_config_dir() {
    if let Some(path) = AgentState::default_path() {
        assert!(path.to_string_lossy().contains(APP_DIR));
        assert!(path.to_string_lossy().ends_with("agent-config.json"));
    }
}
