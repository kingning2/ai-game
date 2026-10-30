//! voices.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

#[test]
fn ids_are_unique_and_assignment_matches_decision() {
    let mut ids: Vec<_> = VOICES.iter().map(|v| v.id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), VOICES.len(), "音色 id 不得重复");

    for voice in VOICES.iter() {
        assert!(
            (0..103).contains(&voice.sid),
            "sid {} 超出 kokoro 音色范围 0-102",
            voice.sid
        );
    }

    // 主持人 = 用户选定的 Grok 4 音色（kokoro sid 62），选手 = 通义千问音色（sid 58）
    let host_sid = find_by_label("主持人").map(|v| v.sid);
    assert_eq!(host_sid, Some(62), "主持人应使用 Grok 音色（sid 62）");
    for voice in VOICES.iter() {
        if voice.label != "主持人" {
            assert_eq!(
                Some(voice.sid),
                Some(58),
                "选手 {} 应用通义千问音色",
                voice.label
            );
        }
    }
}

#[test]
fn labels_cover_players_and_host() {
    for label in [
        "主持人",
        "Claude Opus 5.5",
        "Kimi K3.1 Pro",
        "Gemini 4 Argon",
        "GPT-6.1",
        "Grok 4.7",
        "MiMo V2.6",
        "Qwen3.8 Max",
        "DeepSeek V4.1",
        "GLM-5.3",
        "MiniMax M3.1",
        "豆包 Seed 2.1",
        "Mistral Large 4",
    ] {
        assert!(find_by_label(label).is_some(), "缺少音色：{label}");
    }
}

#[test]
fn speeds_are_sane() {
    for voice in VOICES.iter() {
        assert!(
            (0.5..=2.0).contains(&voice.speed),
            "{} 语速 {} 超界",
            voice.id,
            voice.speed
        );
    }
}

#[test]
fn find_works() {
    assert_eq!(find("zf_001").map(|v| v.label), Some("豆包 Seed 2.1"));
    assert!(find("nope").is_none());
}
