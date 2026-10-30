#![allow(clippy::print_stdout)] // 临时探测工具，stdout 即输出形态
//! 临时探测工具：对候选音色逐个真实合成，验证当前哪些能出音频。
//! 用法：cargo run -p tts --example probe_voices
use edge_tts_rust::{EdgeTtsClient, SpeakOptions};

const CANDIDATES: [&str; 14] = [
    "zh-CN-XiaoxiaoNeural",
    "zh-CN-XiaoyiNeural",
    "zh-CN-YunjianNeural",
    "zh-CN-YunxiNeural",
    "zh-CN-YunxiaNeural",
    "zh-CN-YunyangNeural",
    "zh-CN-liaoning-XiaobeiNeural",
    "zh-CN-shaanxi-XiaoniNeural",
    "zh-CN-YunyeNeural",
    "zh-CN-YunhongNeural",
    "zh-CN-YunzeNeural",
    "zh-CN-XiaochenNeural",
    "zh-CN-XiaohanNeural",
    "zh-CN-XiaomoNeural",
];

#[tokio::main]
async fn main() {
    for voice in CANDIDATES {
        let client = match EdgeTtsClient::new() {
            Ok(c) => c,
            Err(err) => {
                println!("{voice} | 客户端创建失败: {err}");
                continue;
            }
        };
        let result = client
            .synthesize(
                "你好，这是一次音色可用性探测。",
                SpeakOptions {
                    voice: voice.to_string(),
                    rate: "+0%".into(),
                    output_format: "audio-24khz-48kbitrate-mono-mp3".into(),
                    ..SpeakOptions::default()
                },
            )
            .await;
        match result {
            Ok(audio) => println!("{voice} | OK | {} 字节", audio.audio.len()),
            Err(err) => println!("{voice} | FAIL | {err}"),
        }
    }
}
