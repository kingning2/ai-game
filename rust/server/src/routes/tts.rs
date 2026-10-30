//! 本地/云端语音合成端点：状态查询 / 模型安装 / 台词合成。
//!
//! - `GET  /api/tts/status`：装配状态（installed / phase / 引擎加载）+ 音色表 + 豆包可用性
//! - `POST /api/tts/setup`：后台安装模型（147MB，首次约 1-2 分钟；幂等，重复调用去重）
//! - `POST /api/tts/speak`：`{ voice, text, engine? }` → 音频字节流
//!   - 默认本地引擎（kokoro，audio/wav）：voice 传音色 id 如 `zf_001`，
//!     或选手名如 `GPT-6.1`；主持人传 `主持人`；首次 speak 前必须 setup
//!   - `engine: "doubao"`（豆包官方音色，audio/aac）：voice 传音色简称（如 `taozi`）
//!     或完整音色 id；需要服务端配置 `DOUBAO_COOKIE`，否则 400 引导

use crate::error::{ApiError, ApiResponse, ApiResult, AppError};
use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use state::AppState;
use tracing::warn;
use tts::voices;

/// GET /api/tts/status 响应
#[derive(Serialize, utoipa::ToSchema)]
pub struct TtsStatusDto {
    /// 模型资产是否已安装
    pub installed: bool,
    /// 装配阶段：idle / installing / ready / failed
    pub phase: String,
    /// 引擎是否已加载（首次合成前为 false）
    pub engine_loaded: bool,
    /// 模型缓存目录（绝对路径）
    pub cache_dir: String,
    /// 豆包云端引擎是否可用（取决于服务端是否配置了 DOUBAO_COOKIE）
    pub doubao_available: bool,
    /// 可用音色（12 选手 + 主持人）
    pub voices: Vec<TtsVoiceDto>,
}

/// 音色项
#[derive(Serialize, utoipa::ToSchema)]
pub struct TtsVoiceDto {
    /// 音色 id（speak 的 voice 字段）
    pub id: String,
    /// 展示名（选手名 / 主持人）
    pub label: String,
}

/// POST /api/tts/speak 请求体
#[derive(Deserialize, utoipa::ToSchema)]
pub struct TtsSpeakRequest {
    /// 音色 id（`zf_001`）或选手名（`豆包` / `主持人`）；豆包引擎传简称（`taozi`）
    pub voice: String,
    /// 要朗读的台词（≤500 字符）
    pub text: String,
    /// 引擎：省略或 `local` 走本地 kokoro（WAV）；`doubao` 走豆包官方云端（AAC）
    #[serde(default)]
    pub engine: Option<String>,
}

/// GET /api/tts/status
#[utoipa::path(
    get,
    path = "/api/tts/status",
    tag = "tts",
    operation_id = "get_tts_status",
    responses(
        (status = 200, description = "TTS 装配状态与音色表", body = ApiResponse<TtsStatusDto>)
    )
)]
pub async fn get_status(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<TtsStatusDto>>> {
    let status = state.tts.status();
    Ok(ApiResponse::ok(TtsStatusDto {
        installed: status.installed,
        phase: status.phase.as_str().to_string(),
        engine_loaded: status.engine_loaded,
        cache_dir: status.cache_dir,
        doubao_available: state.config.doubao_cookie.is_some(),
        voices: voices::VOICES
            .iter()
            .map(|voice| TtsVoiceDto {
                id: voice.id.to_string(),
                label: voice.label.to_string(),
            })
            .collect(),
    }))
}

/// POST /api/tts/setup
#[utoipa::path(
    post,
    path = "/api/tts/setup",
    tag = "tts",
    operation_id = "setup_tts",
    responses(
        (status = 200, description = "已触发后台安装（返回触发时状态；用 status 轮询 phase 直到 ready/failed）", body = ApiResponse<TtsStatusDto>),
    )
)]
pub async fn setup(State(state): State<AppState>) -> ApiResult<Json<ApiResponse<TtsStatusDto>>> {
    // 已安装 → setup 内部幂等跳过；未安装 → 后台下载，接口立即返回
    state.tts.setup_in_background();
    get_status(State(state)).await
}

/// POST /api/tts/speak
#[utoipa::path(
    post,
    path = "/api/tts/speak",
    tag = "tts",
    operation_id = "speak",
    request_body = TtsSpeakRequest,
    responses(
        (status = 200, description = "音频字节流：本地引擎 audio/wav，豆包引擎 audio/aac（按请求 format）", content_type = "audio/wav"),
        (status = 404, description = "未知音色（仅本地引擎）", body = ApiResponse<String>),
        (status = 409, description = "本地引擎模型未安装，先 POST /api/tts/setup", body = ApiResponse<String>),
        (status = 400, description = "text 为空/超长，或豆包引擎未配置 DOUBAO_COOKIE", body = ApiResponse<String>),
        (status = 503, description = "豆包云端服务异常（连接/协议/服务端错误）", body = ApiResponse<String>),
    )
)]
pub async fn speak(
    State(state): State<AppState>,
    Json(request): Json<TtsSpeakRequest>,
) -> Result<axum::response::Response, ApiError> {
    if request.engine.as_deref() == Some("doubao") {
        return speak_doubao(&state, request.voice, &request.text).await;
    }
    if request.engine.as_deref() == Some("edge") {
        return speak_edge(&request.voice, &request.text).await;
    }

    let voice = voices::find(&request.voice)
        .or_else(|| voices::find_by_label(&request.voice))
        .ok_or_else(|| AppError::NotFound(format!("未知音色：{}", request.voice)))?;

    let bytes = state
        .tts
        .synthesize(voice, &request.text)
        .await
        .map_err(tts_error)?;

    Ok((StatusCode::OK, [(header::CONTENT_TYPE, "audio/wav")], bytes).into_response())
}

/// Edge 免费接口分支：观战语音播报专用（狼人杀座位音色分配的合成端）
async fn speak_edge(voice: &str, text: &str) -> Result<axum::response::Response, ApiError> {
    let bytes = tts::edge::synthesize(voice, text, tts::edge::DEFAULT_SPEED)
        .await
        .map_err(tts_error)?;
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, "audio/mpeg")], bytes).into_response())
}

/// 豆包云端引擎分支：官方音色（协议逆向自社区项目 callmerio/doubao-tts）
async fn speak_doubao(
    state: &AppState,
    voice: String,
    text: &str,
) -> Result<axum::response::Response, ApiError> {
    let cookie = state.config.doubao_cookie.as_deref().ok_or_else(|| {
        AppError::BadRequest(
            "豆包引擎不可用：服务端未配置 DOUBAO_COOKIE（浏览器登录 doubao.com 后从 DevTools 复制 Cookie，设置到环境变量）"
                .to_string(),
        )
    })?;

    let config = tts::doubao::DoubaoConfig {
        speaker: tts::doubao::resolve_speaker(&voice).to_string(),
        ..tts::doubao::DoubaoConfig::new(cookie)
    };
    let result = tts::doubao::synthesize(&config, text)
        .await
        .map_err(tts_error)?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, config.mime())],
        result.audio,
    )
        .into_response())
}

/// TtsError → ApiError：5xx 分支记日志（完整原因只进日志，防信息泄露）
fn tts_error(err: tts::TtsError) -> ApiError {
    let app = match &err {
        tts::TtsError::NotReady => {
            AppError::Conflict("TTS 模型未安装，请先 POST /api/tts/setup".to_string())
        }
        tts::TtsError::InvalidInput(_) => AppError::BadRequest(err.to_string()),
        tts::TtsError::Ws(_) => {
            // 上游（豆包）失败：完整原因只进日志；Cookie 过期/接口变动时看日志排查
            warn!(error = %err, "豆包云端 TTS 失败");
            AppError::ServiceUnavailable
        }
        tts::TtsError::Asset(_) => AppError::internal(format!("tts asset: {err}")),
        tts::TtsError::Engine(_) | tts::TtsError::Io(_) | tts::TtsError::Http(_) => {
            AppError::internal(format!("tts engine: {err}"))
        }
    };
    ApiError(app)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/tts/status", get(get_status))
        .route("/api/tts/setup", post(setup))
        .route("/api/tts/speak", post(speak))
}

#[cfg(test)]
#[path = "tts_tests.rs"]
mod tests;
