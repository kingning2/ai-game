//! 豆包 TTS 冒烟示例：用官方音色合成一句话，经 ffmpeg 转码输出 mp3。
//!
//! 豆包云端只出 aac/wav/pcm/ogg_opus（不支持 mp3），mp3 由本示例调用 ffmpeg 转码。
//! 鉴权走浏览器登录 doubao.com 的 Cookie（至少含 sessionid）：
//!
//! ```bash
//! DOUBAO_COOKIE='sessionid=xxxx' cargo run -p tts --example doubao_demo
//! ```
//!
//! 注意：同 IP 高频请求会触发豆包风控（网关报 710020702"系统错误"），稍等片刻重试即可。

use tts::doubao::{self, DoubaoConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cookie = std::env::var("DOUBAO_COOKIE")
        .map_err(|_| "缺少 DOUBAO_COOKIE 环境变量（浏览器登录 doubao.com 后复制 Cookie）")?;

    let config = DoubaoConfig::new(cookie);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let result = runtime.block_on(doubao::synthesize(
        &config,
        "各位玩家晚上好，欢迎来到狼人杀。天黑请闭眼，祝你好运！",
    ))?;

    let raw = "doubao-demo.aac";
    std::fs::write(raw, &result.audio)?;
    println!(
        "合成完成：{raw} 共 {} 字节，{} 句",
        result.audio.len(),
        result.sentences.len()
    );

    // AAC → MP3（qscale 4 ≈ 190kbps VBR，人声足够）
    let mp3 = "doubao-demo.mp3";
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-i",
            raw,
            "-codec:a",
            "libmp3lame",
            "-qscale:a",
            "4",
            mp3,
        ])
        .status()
        .map_err(|err| format!("调用 ffmpeg 失败（请确认已安装并加入 PATH）：{err}"))?;
    if !status.success() {
        return Err("ffmpeg 转码失败".into());
    }
    std::fs::remove_file(raw)?;

    let size = std::fs::metadata(mp3)?.len();
    println!("已生成 {mp3}（{size} 字节，音色 {}）", config.speaker);
    for sentence in &result.sentences {
        println!("句子：{sentence}");
    }
    Ok(())
}
