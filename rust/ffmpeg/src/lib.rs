//! 对局录制核心 —— [`Recorder`]：webm 块流 → ffmpeg 子进程 stdin → 落盘。
//!
//! 画面只存在于前端（对局舞台是浏览器渲染的），server 自己无画面可抓。
//! 前端 getDisplayMedia 采集标签页 → MediaRecorder 出 webm chunk → 经 WS 送到
//! server，本 crate 把块流喂给 `ffmpeg -i pipe:0 -c copy`，转存为
//! `<dir>/<label>-<ts>.webm`。连接断开 = 录制结束：stdin EOF 让 ffmpeg 写完
//! 容器索引后自然退出，文件即完整可用。
//!
//! 为什么 -c copy：MediaRecorder 已在浏览器侧完成编码（vp9/opus），转存零转码开销。
//!
//! 职责边界：本 crate 只管 ffmpeg 进程与文件；WebSocket 收流与对客户端的
//! 消息格式归 server 路由层（`routes/record.rs`）。

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin, Command};

/// 录制错误（路由层据此对客户端回 error 消息）
#[derive(Debug, thiserror::Error)]
pub enum FfmpegError {
    /// 输出目录创建失败
    #[error("录制目录创建失败: {0}")]
    Directory(#[source] std::io::Error),
    /// ffmpeg 启动失败（未安装或不在 PATH）
    #[error("ffmpeg 启动失败: {0}")]
    Spawn(#[source] std::io::Error),
    /// ffmpeg 进程收尾失败（wait 出错）
    #[error("ffmpeg 收尾失败: {0}")]
    Wait(#[source] std::io::Error),
    /// ffmpeg 异常退出（非零状态码）
    #[error("ffmpeg 异常退出: {status}")]
    Failed { status: std::process::ExitStatus },
}

/// 一路录制会话：持有 ffmpeg 子进程与其 stdin，直至 [`Recorder::finish`]。
pub struct Recorder {
    child: Child,
    stdin: Option<ChildStdin>,
    out_path: PathBuf,
}

impl Recorder {
    /// 启动 ffmpeg 并开始接收块流。`dir` 不存在会自动创建。
    ///
    /// `label` 经 [`safe_name`] 安全化作为文件名主体；同秒多次启动由
    /// 时间戳区分（同 label 同秒覆盖属可接受的边缘情况，录制是低频操作）。
    pub async fn start(dir: &Path, label: &str) -> Result<Self, FfmpegError> {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(FfmpegError::Directory)?;
        let out_path = dir.join(format!(
            "{}-{}.webm",
            safe_name(label),
            unix_now()
        ));

        // stdin 喂 webm（matroska 流），-c copy 直存；输出强制 webm 容器
        let mut child = Command::new("ffmpeg")
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
            .map_err(FfmpegError::Spawn)?;
        let stdin = child.stdin.take();
        Ok(Self {
            child,
            stdin,
            out_path,
        })
    }

    /// 落盘文件路径（启动后即可取，录制中也可用）
    pub fn output_path(&self) -> &Path {
        &self.out_path
    }

    /// 写入一个 webm 块。返回 `false` = ffmpeg 已退出（异常），调用方应停止收流。
    pub async fn write_chunk(&mut self, data: &[u8]) -> bool {
        match self.stdin.as_mut() {
            Some(stdin) => stdin.write_all(data).await.is_ok(),
            None => false,
        }
    }

    /// 结束录制：关 stdin（EOF → ffmpeg 写容器尾自然退出）并等待进程退出。
    /// 返回落盘路径；ffmpeg 异常退出时返回 [`FfmpegError`]（文件可能不完整，
    /// 由调用方决定是否告知客户端）。
    pub async fn finish(mut self) -> Result<PathBuf, FfmpegError> {
        self.stdin = None; // drop ChildStdin → EOF
        let status = self.child.wait().await.map_err(FfmpegError::Wait)?;
        if status.success() {
            Ok(self.out_path)
        } else {
            Err(FfmpegError::Failed { status })
        }
    }
}

/// 文件名安全化：剥路径分隔与空白（防 `../` 与非法字符），上限 48 字符
pub fn safe_name(raw: &str) -> String {
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

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
