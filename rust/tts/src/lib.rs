//! 本地语音合成：kokoro 模型进程内推理。
//!
//! - [`assets`]：模型包下载/解包（缓存于本地数据目录，全部完成后离线可用）
//! - [`voices`]：12 名选手 + 主持人的音色表
//! - [`TtsService`]：对外服务门面——`setup()` 装模型，`synthesize()` 出 WAV 字节
//!
//! 引擎为 `sherpa-onnx` 官方 Rust crate（默认静态链接，build.rs 首次构建时
//! 自动下载对应平台原生库）；模型常驻进程内，只加载一次。

pub mod assets;
pub mod doubao;
pub mod voices;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsKokoroModelConfig,
    OfflineTtsModelConfig,
};
use tokio::sync::Mutex as AsyncMutex;

/// 单次合成文本上限（发言面板单条台词远短于此）
const MAX_TEXT_CHARS: usize = 500;

/// 装配阶段（接口展示用）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupPhase {
    /// 未安装（或未跑过 setup）
    Idle,
    /// 正在下载/解包
    Installing,
    /// 已就绪
    Ready,
    /// 上次 setup 失败（可重试）
    Failed,
}

impl SetupPhase {
    fn from_u8(raw: u8) -> Self {
        match raw {
            1 => Self::Installing,
            2 => Self::Ready,
            3 => Self::Failed,
            _ => Self::Idle,
        }
    }

    /// 接口字符串：idle / installing / ready / failed
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Installing => "installing",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

/// 运行状态快照（server 组 DTO 用）
#[derive(Debug, Clone)]
pub struct TtsStatus {
    /// 模型资产是否已安装（`.ok` 标记存在）
    pub installed: bool,
    /// 装配阶段
    pub phase: SetupPhase,
    /// 引擎是否已加载（首次合成前为 false）
    pub engine_loaded: bool,
    /// 缓存目录绝对路径
    pub cache_dir: String,
}

#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    #[error("TTS 模型未安装，请先触发安装（POST /api/tts/setup）")]
    NotReady,
    #[error("模型资产异常：{0}")]
    Asset(String),
    #[error("合成引擎错误：{0}")]
    Engine(String),
    #[error("输入非法：{0}")]
    InvalidInput(String),
    #[error("豆包云端服务异常：{0}")]
    Ws(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

/// 本地 TTS 服务：模型常驻内存，串行合成（内部互斥，天然排队）
#[derive(Clone)]
pub struct TtsService {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for TtsService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TtsService")
            .field("cache_dir", &self.inner.cache_dir)
            .field("phase", &self.inner.phase.load(Ordering::Relaxed))
            .field("engine_loaded", &self.status().engine_loaded)
            .finish_non_exhaustive()
    }
}

struct Inner {
    cache_dir: PathBuf,
    /// 常驻引擎（首次合成时懒加载；跨 await 不持锁，用 std 锁）
    engine: Mutex<Option<OfflineTts>>,
    /// setup 并发去重
    setup_lock: AsyncMutex<()>,
    /// 装配阶段（原子，供 status 读取）
    phase: AtomicU8,
}

impl TtsService {
    /// 指定缓存目录创建服务（不触发下载）
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            inner: Arc::new(Inner {
                cache_dir: cache_dir.into(),
                engine: Mutex::new(None),
                setup_lock: AsyncMutex::new(()),
                phase: AtomicU8::new(0),
            }),
        }
    }

    /// 默认缓存目录（平台本地数据目录）
    pub fn with_default_dir() -> Self {
        Self::new(assets::default_cache_dir())
    }

    /// 缓存目录
    pub fn cache_dir(&self) -> &Path {
        &self.inner.cache_dir
    }

    /// 当前状态快照
    pub fn status(&self) -> TtsStatus {
        let installed = assets::installed(&self.inner.cache_dir);
        let phase = SetupPhase::from_u8(self.inner.phase.load(Ordering::Relaxed));
        // 进程重启后内存 phase 归零：资产已装视为 ready，避免“已装却显示 idle”的歧义
        let phase = if installed && matches!(phase, SetupPhase::Idle) {
            SetupPhase::Ready
        } else {
            phase
        };
        TtsStatus {
            installed,
            phase,
            engine_loaded: self
                .inner
                .engine
                .lock()
                .map(|guard| guard.is_some())
                .unwrap_or(false),
            cache_dir: self.inner.cache_dir.display().to_string(),
        }
    }

    /// 安装模型资产（幂等：已安装直接跳过；并发调用只执行一次）
    pub async fn setup(&self) -> Result<(), TtsError> {
        let _guard = self.inner.setup_lock.lock().await;
        if assets::installed(&self.inner.cache_dir) {
            self.inner.phase.store(2, Ordering::Relaxed);
            return Ok(());
        }

        self.inner.phase.store(1, Ordering::Relaxed);
        let result = assets::ensure(&self.inner.cache_dir).await;
        match &result {
            Ok(()) => self.inner.phase.store(2, Ordering::Relaxed),
            Err(_) => self.inner.phase.store(3, Ordering::Relaxed),
        }
        result
    }

    /// 后台安装（fire-and-forget；进度经 `status()` 轮询）
    pub fn setup_in_background(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            if let Err(err) = service.setup().await {
                tracing::warn!(error = %err, "TTS 模型安装失败");
            }
        });
    }

    /// 确保模型就绪并加载引擎，随后串行合成，返回 WAV 字节
    pub async fn synthesize(
        &self,
        voice: &voices::VoiceSpec,
        text: &str,
    ) -> Result<Vec<u8>, TtsError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(TtsError::InvalidInput("text 不能为空".to_string()));
        }
        if text.chars().count() > MAX_TEXT_CHARS {
            return Err(TtsError::InvalidInput(format!(
                "text 超过 {MAX_TEXT_CHARS} 字符"
            )));
        }
        if !assets::installed(&self.inner.cache_dir) {
            return Err(TtsError::NotReady);
        }

        // 合成是 CPU 密集任务：丢到阻塞线程池，避免卡 runtime
        let inner = self.inner.clone();
        let text = text.to_string();
        let sid = voice.sid;
        let speed = voice.speed;
        tokio::task::spawn_blocking(move || synthesize_blocking(&inner, &text, sid, speed))
            .await
            .map_err(|err| TtsError::Engine(err.to_string()))?
    }
}

/// 阻塞线程里的完整合成流程：懒加载引擎 → 推理 → 写临时 WAV → 读字节
fn synthesize_blocking(
    inner: &Inner,
    text: &str,
    sid: i32,
    speed: f32,
) -> Result<Vec<u8>, TtsError> {
    let mut guard = inner
        .engine
        .lock()
        .map_err(|err| TtsError::Engine(format!("引擎锁中毒：{err}")))?;

    if guard.is_none() {
        let model_dir = assets::model_dir(&inner.cache_dir);
        let engine = OfflineTts::create(&engine_config(&model_dir)?).ok_or_else(|| {
            TtsError::Engine("引擎初始化失败：模型文件/目录无效或与当前平台不兼容".to_string())
        })?;
        tracing::info!(
            sample_rate = engine.sample_rate(),
            num_speakers = engine.num_speakers(),
            "TTS 引擎加载完成"
        );
        *guard = Some(engine);
    }
    let engine = guard
        .as_ref()
        .ok_or_else(|| TtsError::Engine("引擎内部状态异常：加载后丢失".to_string()))?;

    let audio = engine
        .generate_with_config(
            text,
            &GenerationConfig {
                sid,
                speed,
                ..GenerationConfig::default()
            },
            None::<fn(&[f32], f32) -> bool>,
        )
        .ok_or_else(|| TtsError::Engine("合成失败：文本无法转语音".to_string()))?;

    // GeneratedAudio::save 写 WAV 文件；用临时文件中转再读字节，读后即删
    let wav_path = inner.cache_dir.join(format!(
        "speak-{}.wav",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    if !audio.save(wav_path.to_string_lossy().as_ref()) {
        return Err(TtsError::Engine("WAV 写出失败".to_string()));
    }
    let bytes = std::fs::read(&wav_path)?;
    let _ = std::fs::remove_file(&wav_path);
    Ok(bytes)
}

/// kokoro 引擎配置（相对模型目录；中文词库 + jieba 分词字典 + 英文词库）
fn engine_config(model_dir: &Path) -> Result<OfflineTtsConfig, TtsError> {
    let must = |name: &str| -> Result<PathBuf, TtsError> {
        let path = model_dir.join(name);
        if path.exists() {
            Ok(path)
        } else {
            Err(TtsError::Asset(format!(
                "缺少 {name}（{}）——模型目录不完整，删除缓存目录后重新 setup",
                path.display()
            )))
        }
    };

    // 官方包内为 model.int8.onnx；兼容无量化后缀的 model.onnx
    let model = if model_dir.join("model.int8.onnx").is_file() {
        model_dir.join("model.int8.onnx")
    } else {
        must("model.onnx")?
    };
    let voices_file = must("voices.bin")?;
    let tokens = must("tokens.txt")?;
    let data_dir = must("espeak-ng-data")?;
    let dict_dir = must("dict")?;
    let lexicon_us = must("lexicon-us-en.txt")?;
    let lexicon_zh = must("lexicon-zh.txt")?;

    Ok(OfflineTtsConfig {
        model: OfflineTtsModelConfig {
            kokoro: OfflineTtsKokoroModelConfig {
                model: Some(model.to_string_lossy().into_owned()),
                voices: Some(voices_file.to_string_lossy().into_owned()),
                tokens: Some(tokens.to_string_lossy().into_owned()),
                data_dir: Some(data_dir.to_string_lossy().into_owned()),
                dict_dir: Some(dict_dir.to_string_lossy().into_owned()),
                lexicon: Some(format!(
                    "{},{}",
                    lexicon_zh.to_string_lossy(),
                    lexicon_us.to_string_lossy()
                )),
                ..OfflineTtsKokoroModelConfig::default()
            },
            num_threads: 2,
            debug: false,
            ..OfflineTtsModelConfig::default()
        },
        ..OfflineTtsConfig::default()
    })
}
