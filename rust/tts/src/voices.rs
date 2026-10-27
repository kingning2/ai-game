//! 音色表：12 名 Agent 选手 + 主持人各分配一个 kokoro 音色（sid）。
//!
//! 声音分配（产品决策）：主持人用 zm_013（Grok 4）的音色，
//! 12 名选手统一用 zm_009（通义千问）的音色；
//! id/label 保留每个选手的身份映射，接口层仍按 id 或 label 查询。

/// 一个音色的完整描述：引擎参数（sid/speed）+ 前端展示信息（id/label）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoiceSpec {
    /// 音色标识（接口用，如 `zf_001`）
    pub id: &'static str,
    /// 展示名（前端按此对齐选手名；主持人固定为"主持人"）
    pub label: &'static str,
    /// 引擎音色编号（kokoro sid，0-102）
    pub sid: i32,
    /// 语速（0.5-2.0，越大越快）
    pub speed: f32,
}

/// 全部内置音色（含主持人）
pub const VOICES: &[VoiceSpec] = &[
    VoiceSpec {
        id: "zm_100",
        label: "主持人",
        sid: 62,
        speed: 0.95,
    },
    VoiceSpec {
        id: "zf_001",
        label: "豆包 Seed 2.1",
        sid: 58,
        speed: 1.0,
    },
    VoiceSpec {
        id: "zf_002",
        label: "Mistral Large 4",
        sid: 58,
        speed: 1.0,
    },
    VoiceSpec {
        id: "zf_003",
        label: "GLM-5.3",
        sid: 58,
        speed: 1.0,
    },
    VoiceSpec {
        id: "zf_017",
        label: "DeepSeek V4.1",
        sid: 58,
        speed: 1.0,
    },
    VoiceSpec {
        id: "zm_009",
        label: "Qwen3.8 Max",
        sid: 58,
        speed: 1.0,
    },
    VoiceSpec {
        id: "zf_018",
        label: "Gemini 4 Argon",
        sid: 58,
        speed: 1.0,
    },
    VoiceSpec {
        id: "zf_019",
        label: "Kimi K3.1 Pro",
        sid: 58,
        speed: 1.0,
    },
    VoiceSpec {
        id: "zm_010",
        label: "Claude Opus 5.5",
        sid: 58,
        speed: 1.0,
    },
    VoiceSpec {
        id: "zf_021",
        label: "MiniMax M3.1",
        sid: 58,
        speed: 1.0,
    },
    VoiceSpec {
        id: "zm_011",
        label: "Grok 4.7",
        sid: 58,
        speed: 1.0,
    },
    VoiceSpec {
        id: "zm_012",
        label: "GPT-6.1",
        sid: 58,
        speed: 1.0,
    },
    VoiceSpec {
        id: "zm_013",
        label: "MiMo V2.6",
        sid: 58,
        speed: 1.0,
    },
];

/// 按 id 查音色
pub fn find(id: &str) -> Option<&'static VoiceSpec> {
    VOICES.iter().find(|voice| voice.id == id)
}

/// 按展示名（选手名/主持人）查音色
pub fn find_by_label(label: &str) -> Option<&'static VoiceSpec> {
    VOICES.iter().find(|voice| voice.label == label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_assignment_matches_decision() {
        let mut ids: Vec<_> = VOICES.iter().map(|v| v.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), VOICES.len(), "音色 id 不得重复");

        for voice in VOICES {
            assert!(
                (0..103).contains(&voice.sid),
                "sid {} 超出 kokoro 音色范围 0-102",
                voice.sid
            );
        }

        // 主持人 = 用户选定的 Grok 4 音色（kokoro sid 62），选手 = 通义千问音色（sid 58）
        let host_sid = find_by_label("主持人").map(|v| v.sid);
        assert_eq!(host_sid, Some(62), "主持人应使用 Grok 音色（sid 62）");
        for voice in VOICES {
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
        for voice in VOICES {
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
}
