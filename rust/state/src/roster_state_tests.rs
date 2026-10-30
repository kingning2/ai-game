//! roster_state.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use crate::agent_state::tests_support::temp_path;

fn global() -> AgentConfig {
    AgentConfig {
        openai: ProviderSettings {
            base_url: "https://global.example.com/v1".into(),
            api_key: "sk-global".into(),
            model: "global-model".into(),
        },
        anthropic: ProviderSettings {
            base_url: "https://anthropic.example.com".into(),
            api_key: String::new(),
            model: "claude-global".into(),
        },
        ..AgentConfig::default()
    }
}

#[test]
fn resolve_without_override_returns_global() {
    let roster = RosterState::ephemeral();
    let resolved = roster.resolve("豆包 Seed 2.1", &global());
    assert_eq!(resolved, global());
}

#[test]
fn resolve_merges_field_level_patch() {
    let roster = RosterState::ephemeral();
    roster
        .set_override(
            "豆包 Seed 2.1",
            Some(PlayerOverride {
                openai: Some(SettingsPatch {
                    model: Some("doubao-seed-1-6".into()),
                    ..SettingsPatch::default()
                }),
                max_tokens: Some(8192),
                ..PlayerOverride::default()
            }),
        )
        .unwrap();

    let resolved = roster.resolve("豆包 Seed 2.1", &global());
    // 覆盖的字段生效
    assert_eq!(resolved.openai.model, "doubao-seed-1-6");
    assert_eq!(resolved.max_tokens, 8192);
    // 未覆盖的字段继承全局
    assert_eq!(resolved.openai.base_url, "https://global.example.com/v1");
    assert_eq!(resolved.openai.api_key, "sk-global");
    // 全局原配置不被污染
    assert_eq!(global().openai.model, "global-model");
}

#[test]
fn resolve_applies_provider_switch() {
    let roster = RosterState::ephemeral();
    roster
        .set_override(
            "Kimi K3.1 Pro",
            Some(PlayerOverride {
                provider: Some(ProviderKind::Anthropic),
                ..PlayerOverride::default()
            }),
        )
        .unwrap();

    let resolved = roster.resolve("Kimi K3.1 Pro", &global());
    assert_eq!(resolved.provider, ProviderKind::Anthropic);
    assert_eq!(resolved.anthropic.model, "claude-global");
}

#[test]
fn setting_empty_override_clears_entry() {
    let roster = RosterState::ephemeral();
    roster
        .set_override(
            "豆包 Seed 2.1",
            Some(PlayerOverride {
                max_tokens: Some(1024),
                ..PlayerOverride::default()
            }),
        )
        .unwrap();
    assert!(roster.override_of("豆包 Seed 2.1").is_some());

    roster.set_override("豆包 Seed 2.1", None).unwrap();
    assert!(roster.override_of("豆包 Seed 2.1").is_none());
}

#[test]
fn custom_player_can_be_created_and_removed() {
    let roster = RosterState::ephemeral();
    roster
        .set_override(
            "神秘嘉宾",
            Some(PlayerOverride {
                avatar: Some("🎭".into()),
                openai: Some(SettingsPatch {
                    model: Some("doubao-seed-2-1-pro".into()),
                    ..SettingsPatch::default()
                }),
                ..PlayerOverride::default()
            }),
        )
        .unwrap();
    // 自建选手进入名单，且排在内置 12 人之后
    let names = roster.player_names();
    assert_eq!(names.len(), 13);
    assert_eq!(names.last().unwrap(), "神秘嘉宾");
    assert_eq!(
        roster.override_of("神秘嘉宾").unwrap().avatar.as_deref(),
        Some("🎭")
    );

    // 空名拒绝
    let err = roster
        .set_override("  ", Some(PlayerOverride::default()))
        .unwrap_err();
    assert!(matches!(err, StateError::Invalid(_)));

    // 内置选手不可删，自建可删
    assert!(roster.remove_player("豆包 Seed 2.1").is_err());
    roster.remove_player("神秘嘉宾").unwrap();
    assert!(roster.override_of("神秘嘉宾").is_none());
    assert_eq!(roster.player_names().len(), 12);
}

#[test]
fn persists_and_reloads_roundtrip() {
    let path = temp_path("roster-roundtrip");
    {
        let roster = RosterState::load(path.clone());
        roster
            .set_override(
                "MiniMax M3.1",
                Some(PlayerOverride {
                    openai: Some(SettingsPatch {
                        base_url: Some("https://api.deepseek.com/v1".into()),
                        api_key: Some("sk-r1".into()),
                        model: Some("deepseek-reasoner".into()),
                    }),
                    ..PlayerOverride::default()
                }),
            )
            .unwrap();
    }

    let reloaded = RosterState::load(path.clone());
    let player_override = reloaded.override_of("MiniMax M3.1").expect("覆盖应已落盘");
    assert_eq!(
        player_override.openai.expect("openai 补丁应存在").model,
        Some("deepseek-reasoner".into())
    );
    std::fs::remove_file(&path).ok();
}
