//! 音色表：12 名 Agent 选手 + 主持人各分配一个 kokoro 音色（sid）。
//!
//! 声音分配（产品决策）：主持人用 zm_013（Grok 4）的音色，
//! 12 名选手统一用 zm_009（通义千问）的音色；
//! id/label 保留每个选手的身份映射，接口层仍按 id 或 label 查询。

use std::sync::LazyLock;

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

/// 选手名单（id 顺序即引擎音色表顺序，label 与前端选手名对齐）
const PLAYERS: [(&str, &str); 12] = [
    ("zf_001", "豆包 Seed 2.1"),
    ("zf_002", "Mistral Large 4"),
    ("zf_003", "GLM-5.3"),
    ("zf_017", "DeepSeek V4.1"),
    ("zm_009", "Qwen3.8 Max"),
    ("zf_018", "Gemini 4 Argon"),
    ("zf_019", "Kimi K3.1 Pro"),
    ("zm_010", "Claude Opus 5.5"),
    ("zf_021", "MiniMax M3.1"),
    ("zm_011", "Grok 4.7"),
    ("zm_012", "GPT-6.1"),
    ("zm_013", "MiMo V2.6"),
];

/// 选手统一音色参数：同一 sid（zm_009 音色）、常速
const PLAYER_SID: i32 = 58;

/// 全部内置音色（主持人置首）：选手 12 条 + 主持人 1 条
pub static VOICES: LazyLock<Vec<VoiceSpec>> = LazyLock::new(|| {
    std::iter::once(VoiceSpec {
        id: "zm_100",
        label: "主持人",
        sid: 62,
        speed: 0.95,
    })
    .chain(PLAYERS.iter().map(|&(id, label)| VoiceSpec {
        id,
        label,
        sid: PLAYER_SID,
        speed: 1.0,
    }))
    .collect()
});

/// 按 id 查音色
pub fn find(id: &str) -> Option<&'static VoiceSpec> {
    VOICES.iter().find(|voice| voice.id == id)
}

/// 按展示名（选手名/主持人）查音色
pub fn find_by_label(label: &str) -> Option<&'static VoiceSpec> {
    VOICES.iter().find(|voice| voice.label == label)
}

#[cfg(test)]
#[path = "voices_tests.rs"]
mod tests;
