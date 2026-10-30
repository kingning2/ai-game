//! 对局录制：前端推流（WebSocket 二进制帧）→ ffmpeg 落盘。
//!
//! 画面只存在于前端（对局舞台是浏览器渲染的），server 自己无画面可抓。
//! 前端 getDisplayMedia 采集标签页 → MediaRecorder 出 webm chunk → 本端点收流，
//! `ffmpeg -i pipe:0 -c copy` 转存为 `recordings/<label>-<ts>.webm`。
//! 连接断开 = 录制结束：stdin EOF 让 ffmpeg 写完容器索引后自然退出，文件即完整可用。
//!
//! 为什么 -c copy：MediaRecorder 已在浏览器侧完成编码（vp9/opus），转存零转码开销。

use std::path::PathBuf;
use std::process::Stdio;

use axum::{
    Router,
    extract::{Query, WebSocketUpgrade, ws},
    routing::get,
};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use state::AppState;

/// 录制文件输出目录（server 工作目录下，`.gitignore` 已排除）
const RECORDINGS_DIR: &str = "recordings";

#[derive(Debug, Deserialize)]
struct RecordQuery {
    /// 房间号（也用作文件名主体）
    room: String,
    /// 显示名（缺省用 room）
    label: Option<String>,
}

/// 文件名安全化：剥路径分隔与空白（防 `../` 与非法字符）
fn safe_name(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "-_".contains(c) {
                c
            } else {
                '-'
            }
        })
        .collect();
    cleaned.chars().take(48).collect()
}

/// GET /api/game/record — WebSocket 升级
async fn record_ws(
    ws: WebSocketUpgrade,
    Query(q): Query<RecordQuery>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_stream(socket, q))
}

/// 单连接录制会话：收 webm chunk → ffmpeg stdin → 文件
async fn handle_stream(mut socket: ws::WebSocket, q: RecordQuery) {
    let label = safe_name(q.label.as_deref().unwrap_or(&q.room));

    // ffmpeg 输出文件名：<label>-<unix秒>.webm
    let out_path: PathBuf = [RECORDINGS_DIR, &format!("{label}-{}.webm", unix_now())]
        .iter()
        .collect();
    if let Err(err) = tokio::fs::create_dir_all(RECORDINGS_DIR).await {
        tracing::error!(error = %err, "录制目录创建失败");
        send_error(&mut socket, "录制目录创建失败").await;
        return;
    }

    // stdin 喂 webm（matroska 流），-c copy 直存；输出强制 webm 容器
    let mut child = match Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "matroska",
            "-i",
            "pipe:0",
            "-c",
            "copy",
            "-f",
            "webm",
        ])
        .arg(&out_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            tracing::error!(error = %err, "ffmpeg 启动失败（未安装或不在 PATH）");
            send_error(
                &mut socket,
                "ffmpeg 不可用：请确认 server 所在机器已安装 ffmpeg",
            )
            .await;
            return;
        }
    };
    tracing::info!(room = %q.room, file = %out_path.display(), "录制开始");

    // 收流循环：二进制帧写 stdin；Close/stop 文本 = 录制结束
    let mut some_stdin = child.stdin.take();
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(ws::Message::Binary(data)) => {
                if let Some(stdin) = some_stdin.as_mut()
                    && stdin.write_all(&data).await.is_err()
                {
                    break; // ffmpeg 已退出（异常）：停止收流
                }
            }
            Ok(ws::Message::Text(_)) => break, // stop 控制消息
            Ok(_) => {}
            Err(_) => break, // 前端断开（关页/刷新）= 录制结束
        }
    }
    drop(some_stdin); // stdin EOF → ffmpeg 写容器尾 → 自然退出

    match child.wait().await {
        Ok(status) => {
            tracing::info!(file = %out_path.display(), status = %status, "录制完成");
            let _ = socket
                .send(ws::Message::Text(
                    format!("{{\"type\":\"saved\",\"file\":\"{}\"}}", out_path.display()).into(),
                ))
                .await;
        }
        Err(err) => tracing::error!(error = %err, "ffmpeg 收尾失败"),
    }
}

async fn send_error(socket: &mut ws::WebSocket, message: &str) {
    let _ = socket
        .send(ws::Message::Text(
            serde_json::json!({ "type": "error", "message": message })
                .to_string()
                .into(),
        ))
        .await;
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/game/record", get(record_ws))
}

#[cfg(test)]
#[path = "record_tests.rs"]
mod tests;
