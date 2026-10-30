//! Edge TTS 引擎：微软 Edge Read Aloud 免费接口（联网，MP3 输出，无需 API Key）。
//!
//! 音色池为该接口当前暴露的全部中文音色精选（8 女声 + 4 男声），
//! 含普通话 / 东北 / 陕西 / 粤语 / 台湾口音，用于每局给 AI 选手随机分配嗓音。

use crate::TtsError;
use edge_tts_rust::{EdgeTtsClient, SpeakOptions};

/// MP3 输出格式（24kHz 单声道，语音场景标准配置）
const MP3_FORMAT: &str = "audio-24khz-48kbitrate-mono-mp3";

/// 默认语速（倍率，1.0 = 原速）：中文 neural 默认偏慢，观战场景提 15% 更紧凑
pub const DEFAULT_SPEED: f32 = 1.15;

/// 语速倍率 → SSML rate（如 1.15 → "+15%"）
fn rate_of(speed: f32) -> String {
    let pct = ((speed - 1.0) * 100.0).round() as i32;
    if pct >= 0 {
        format!("+{pct}%")
    } else {
        format!("{pct}%")
    }
}

/// 音色池：Edge TTS 免费接口的全部**普通话**音色（实测共 6 个：2 女 4 男；
/// 粤语/东北/陕西/台湾腔已按产品要求剔除，其余老普通话音色实测全部 400）。
/// 主持人固定 [`HOST_VOICE`]，[`assign`] 只从其余 5 个里随机，不会撞主持音色。
pub const VOICES: [&str; 6] = [
    "zh-CN-XiaoxiaoNeural", // 女声·温暖（新闻/小说腔）
    "zh-CN-XiaoyiNeural",   // 女声·活泼
    "zh-CN-YunjianNeural",  // 男声·激情（新闻/小说腔）
    "zh-CN-YunxiNeural",    // 男声·阳光
    "zh-CN-YunxiaNeural",   // 男声·可爱
    "zh-CN-YunyangNeural",  // 男声·专业（新闻腔）＝主持人专用
];

/// 主持人固定音色：云扬（专业新闻腔）。
pub const HOST_VOICE: &str = "zh-CN-YunyangNeural";

/// 用 Edge 引擎合成一句台词 → MP3 字节。
///
/// `speed` 语速倍率（1.0 = 原速，允许 0.5–2.0）。
///
/// # Errors
/// 音色不在池内或语速越界 → [`TtsError::InvalidInput`]；联网/协议失败 → [`TtsError::Engine`]
pub async fn synthesize(voice: &str, text: &str, speed: f32) -> Result<Vec<u8>, TtsError> {
    if !VOICES.contains(&voice) {
        return Err(TtsError::InvalidInput(format!("未知 Edge 音色：{voice}")));
    }
    if !(0.5..=2.0).contains(&speed) {
        return Err(TtsError::InvalidInput(format!(
            "语速超出范围（0.5–2.0）：{speed}"
        )));
    }
    // 每次新建 client：发言频率低（秒级一次），WS 握手成本可接受；
    // client 内部连接池跨实例不共享，不值得为省握手引入全局单例
    let client = EdgeTtsClient::new().map_err(|err| TtsError::Engine(err.to_string()))?;
    let result = client
        .synthesize(
            text,
            SpeakOptions {
                voice: voice.to_string(),
                rate: rate_of(speed),
                output_format: MP3_FORMAT.to_string(),
                ..SpeakOptions::default()
            },
        )
        .await
        .map_err(|err| TtsError::Engine(err.to_string()))?;
    Ok(result.audio)
}

/// 每局随机分配玩家音色：以局 id 播种洗牌音色池（不含主持人云扬）取前 `n` 个。
///
/// 纯函数：同一局反复调用结果一致（重启不变），局间 game_id 不同 → 分配不同。
/// 每局随机分配玩家音色：以局 id 播种洗牌音色池（不含主持人云扬），
/// 选手超过池大小时从池头循环复用（相邻选手音色不同，保证与上一轮错开）。
///
/// 纯函数：同一局反复调用结果一致（重启不变），局间 game_id 不同 → 分配不同。
pub fn assign(seed: u64, n: usize) -> Vec<String> {
    use rand::SeedableRng;
    use rand::seq::SliceRandom;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut pool: Vec<String> = VOICES
        .iter()
        .copied()
        .filter(|&v| v != HOST_VOICE)
        .map(str::to_string)
        .collect();
    pool.shuffle(&mut rng);
    (0..n).map(|i| pool[i % pool.len()].clone()).collect()
}

#[cfg(test)]
#[path = "edge_tests.rs"]
mod tests;
