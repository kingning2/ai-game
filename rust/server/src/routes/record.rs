//! 对局录制端点：前端推流（WebSocket 二进制帧）→ [`ffmpeg`] crate 落盘。
//!
//! 进程编排与文件逻辑在 `ffmpeg` crate（[`Recorder`]）；本文件只做两件事：
//! WS 升级 + 收流循环（二进制帧写 Recorder，stop 文本/断开 = 结束），
//! 以及对客户端的 JSON 回执格式（error / saved）。

use std::path::Path;

use axum::{
    Router,
    extract::{Query, WebSocketUpgrade, ws},
    routing::get,
};
use ffmpeg::Recorder;
use serde::Deserialize;

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

/// GET /api/game/record — WebSocket 升级
async fn record_ws(
    ws: WebSocketUpgrade,
    Query(q): Query<RecordQuery>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_stream(socket, q))
}

/// 单连接录制会话：收 webm chunk → Recorder（ffmpeg stdin）→ 文件
async fn handle_stream(mut socket: ws::WebSocket, q: RecordQuery) {
    let label = q.label.as_deref().unwrap_or(&q.room);
    let mut recorder = match Recorder::start(Path::new(RECORDINGS_DIR), label).await {
        Ok(recorder) => recorder,
        Err(err) => {
            tracing::error!(error = %err, room = %q.room, "录制启动失败");
            send_error(&mut socket, &err.to_string()).await;
            return;
        }
    };
    tracing::info!(room = %q.room, file = %recorder.output_path().display(), "录制开始");

    // 收流循环：二进制帧写 stdin；Close/stop 文本 = 录制结束
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(ws::Message::Binary(data)) => {
                if !recorder.write_chunk(&data).await {
                    break; // ffmpeg 已退出（异常）：停止收流
                }
            }
            Ok(ws::Message::Text(_)) => break, // stop 控制消息
            Ok(_) => {}
            Err(_) => break, // 前端断开（关页/刷新）= 录制结束
        }
    }

    match recorder.finish().await {
        Ok(path) => {
            tracing::info!(file = %path.display(), "录制完成");
            let _ = socket
                .send(ws::Message::Text(
                    format!("{{\"type\":\"saved\",\"file\":\"{}\"}}", path.display()).into(),
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

pub fn router() -> Router<AppState> {
    Router::new().route("/api/game/record", get(record_ws))
}
