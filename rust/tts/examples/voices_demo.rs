//! 本地 kokoro 音色试听导出：全部内置音色各合成一句，ffmpeg 转 mp3，
//! 连同试听页 index.html 一起写到 demo-voices/。
//!
//! 用法：cargo run -p tts --example voices_demo

use std::path::Path;
use std::process::Command;

use tts::TtsService;
use tts::voices;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = Path::new("demo-voices");
    std::fs::create_dir_all(out_dir)?;

    let service = TtsService::with_default_dir();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // (文件名, 音色名, 台词)
    let mut entries: Vec<(String, String, String)> = Vec::new();
    for voice in voices::VOICES {
        let sentence = format!("各位玩家，我是{}。天黑请闭眼，狼人请睁眼。", voice.label);
        let wav = runtime.block_on(service.synthesize(voice, &sentence))?;
        let stem = format!("{}-{}", voice.id, voice.label);
        std::fs::write(out_dir.join(format!("{stem}.wav")), &wav)?;
        println!("合成完成：{stem}（{} 字节）", wav.len());
        entries.push((stem, voice.label.to_string(), sentence));
    }

    // WAV → MP3（libmp3lame，qscale 3 ≈ 190kbps VBR）
    for (stem, _, _) in &entries {
        let status = Command::new("ffmpeg")
            .args(["-y", "-loglevel", "error", "-i"])
            .arg(out_dir.join(format!("{stem}.wav")))
            .args(["-codec:a", "libmp3lame", "-qscale:a", "3"])
            .arg(out_dir.join(format!("{stem}.mp3")))
            .status()?;
        if !status.success() {
            return Err(format!("ffmpeg 转码 {stem} 失败").into());
        }
        std::fs::remove_file(out_dir.join(format!("{stem}.wav")))?;
    }

    // 试听页（本地 artifact，系统默认样式即可）
    let mut html = String::from(
        "<!doctype html>\n<html lang=\"zh\">\n<meta charset=\"utf-8\">\n\
         <title>本地音色试听 · kokoro</title>\n\
         <body style=\"font-family:system-ui,sans-serif;max-width:560px;margin:2rem auto\">\n\
         <h2>本地音色试听（kokoro · 13 个）</h2>\n",
    );
    for (stem, label, sentence) in &entries {
        html.push_str(&format!(
            "<section style=\"margin:1.25rem 0\"><b>{label}</b><br>\
             <small>{sentence}</small><br>\
             <audio controls preload=\"none\" src=\"{stem}.mp3\"></audio></section>\n"
        ));
    }
    html.push_str("</body>\n</html>\n");
    std::fs::write(out_dir.join("index.html"), html)?;

    println!(
        "\n全部完成：{} 个音色 → {}\n双击 index.html 即可试听",
        entries.len(),
        out_dir.canonicalize()?.display()
    );
    Ok(())
}
