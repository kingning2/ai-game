//! 模型资产管理：下载 kokoro 模型包（GitHub release tar.bz2）到本地缓存目录。
//!
//! 缓存布局：
//!
//! ```text
//! <cache_dir>/
//! ├── downloads/kokoro-int8-multi-lang-v1_1.tar.bz2   # 下载中间产物
//! └── kokoro-int8-multi-lang-v1_1/                    # 解包后的模型目录
//!     ├── model.onnx
//!     ├── voices.bin
//!     ├── tokens.txt
//!     ├── lexicon-us-en.txt / lexicon-zh.txt
//!     ├── espeak-ng-data/
//!     └── .ok                                         # 解包完整性标记
//! ```
//!
//! 已有 `.ok` 标记时视为安装完成，重复 setup 直接跳过。

use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;

/// 模型包名（同时是 tarball 内的顶层目录名）
pub const MODEL_NAME: &str = "kokoro-int8-multi-lang-v1_1";
/// 模型包下载地址（k2-fsa/sherpa-onnx 官方 release，147MB）
pub const MODEL_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-int8-multi-lang-v1_1.tar.bz2";

/// 解包完整性标记文件：出现即认为模型目录完整可用
const INSTALL_MARKER: &str = ".ok";

/// 默认缓存目录：`%LOCALAPPDATA%/agent-packages/tts`（Windows）/ 对应平台 data_local_dir
pub fn default_cache_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("agent-packages")
        .join("tts")
}

/// 模型目录：`<cache_dir>/<model_name>`
pub fn model_dir(cache_dir: &Path) -> PathBuf {
    cache_dir.join(MODEL_NAME)
}

/// 资产是否已安装（`.ok` 标记存在）
pub fn installed(cache_dir: &Path) -> bool {
    model_dir(cache_dir).join(INSTALL_MARKER).is_file()
}

/// 下载 + 解包模型（已安装则直接返回；下载有 10% 步进日志）
pub async fn ensure(cache_dir: &Path) -> Result<(), super::TtsError> {
    let model = model_dir(cache_dir);
    if installed(cache_dir) {
        tracing::debug!(dir = %model.display(), "TTS 模型已就绪，跳过下载");
        return Ok(());
    }

    tokio::fs::create_dir_all(cache_dir).await?;
    let downloads = cache_dir.join("downloads");
    tokio::fs::create_dir_all(&downloads).await?;
    let archive = downloads.join(format!("{MODEL_NAME}.tar.bz2"));

    if !archive.is_file() {
        download(&archive).await?;
    } else {
        tracing::info!(file = %archive.display(), "发现已下载的模型包，直接解包");
    }

    unpack(&archive, cache_dir).await?;

    // 完整性标记最后写：中途失败重跑时能重新走下载/解包
    tokio::fs::write(model.join(INSTALL_MARKER), b"ok").await?;
    tracing::info!(dir = %model.display(), "TTS 模型安装完成");
    Ok(())
}

/// 流式下载（带 10% 步进进度日志；服务端返回 Content-Length 时才打印百分比）
async fn download(target: &Path) -> Result<(), super::TtsError> {
    tracing::info!(url = MODEL_URL, "开始下载 TTS 模型包");
    let started = std::time::Instant::now();
    let response = reqwest::get(MODEL_URL).await?.error_for_status()?;
    let total = response.content_length();
    let part = target.with_extension("part");

    let mut file = tokio::fs::File::create(&part).await?;
    let mut stream = response.bytes_stream();
    let mut received: u64 = 0;
    let mut next_milestone = 10;

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        received += chunk.len() as u64;
        if let Some(total) = total {
            let percent = received * 100 / total;
            if percent >= next_milestone {
                tracing::info!(percent, "TTS 模型下载中");
                next_milestone += 10;
            }
        }
    }
    file.flush().await?;
    drop(file);
    tokio::fs::rename(&part, target).await?;
    tracing::info!(
        bytes = received,
        elapsed_secs = started.elapsed().as_secs(),
        "TTS 模型包下载完成"
    );
    Ok(())
}

/// 解包 tar.bz2 到缓存目录（阻塞 IO，放独立线程）
async fn unpack(archive: &Path, cache_dir: &Path) -> Result<(), super::TtsError> {
    let archive = archive.to_path_buf();
    let cache_dir = cache_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&archive)?;
        let reader = bzip2::read::MultiBzDecoder::new(std::io::BufReader::new(file));
        let mut tar = tar::Archive::new(reader);
        tar.unpack(&cache_dir)?;
        Ok(())
    })
    .await
    .map_err(|err| super::TtsError::Engine(err.to_string()))?
}
