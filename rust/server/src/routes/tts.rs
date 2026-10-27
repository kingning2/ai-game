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
use logging::warn;
use serde::{Deserialize, Serialize};
use state::AppState;
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
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use state::{AgentState, AppConfig};

    fn test_state() -> AppState {
        AppState::new(AppConfig::default(), AgentState::ephemeral())
    }

    /// 带 DOUBAO_COOKIE 的测试态（仅用于不连网就能到达的分支）
    fn test_state_with_cookie() -> AppState {
        AppState::new(
            AppConfig {
                doubao_cookie: Some("sessionid=test".to_string()),
                ..AppConfig::default()
            },
            AgentState::ephemeral(),
        )
    }

    /// status 接口：固定 13 个音色（主持人 + 12 选手），cache_dir 非空
    /// （installed/phase 不断言——取决于本机是否已装模型）
    #[tokio::test]
    async fn status_lists_all_voices() {
        let app = Router::new().merge(router()).with_state(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tts/status")
                    .body(Body::empty())
                    .expect("请求"),
            )
            .await
            .expect("响应");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("响应体");
        let body: Value = serde_json::from_slice(&bytes).expect("JSON");
        assert_eq!(body["code"], 0);
        let data = &body["data"];
        assert_eq!(data["voices"].as_array().map(Vec::len), Some(13));
        assert_eq!(data["voices"][0]["id"], "zm_100");
        assert_eq!(data["voices"][0]["label"], "主持人");
        assert_eq!(data["voices"][1]["label"], "豆包 Seed 2.1");
        assert!(data["cache_dir"].as_str().is_some_and(|s| !s.is_empty()));
        // 测试态 AppConfig::default() 未配 Cookie → 豆包引擎不可用
        assert_eq!(data["doubao_available"], false);
        assert!(matches!(
            data["phase"].as_str(),
            Some("idle" | "ready" | "failed" | "installing")
        ));
    }

    /// speak 未知音色 → 404（不碰模型资产，无需安装）
    #[tokio::test]
    async fn speak_unknown_voice_is_404() {
        let app = Router::new().merge(router()).with_state(test_state());
        let request = Request::builder()
            .method("POST")
            .uri("/api/tts/speak")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "voice": "不存在的音色", "text": "hi" }).to_string(),
            ))
            .expect("请求");
        let response = app.oneshot(request).await.expect("响应");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("响应体");
        let body: Value = serde_json::from_slice(&bytes).expect("JSON");
        assert_eq!(body["code"], 10004);
    }

    /// speak 空文本 → 400（参数校验先于模型检查）
    #[tokio::test]
    async fn speak_empty_text_is_400() {
        let app = Router::new().merge(router()).with_state(test_state());
        let request = Request::builder()
            .method("POST")
            .uri("/api/tts/speak")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "voice": "豆包 Seed 2.1", "text": "   " }).to_string(),
            ))
            .expect("请求");
        let response = app.oneshot(request).await.expect("响应");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("响应体");
        let body: Value = serde_json::from_slice(&bytes).expect("JSON");
        assert_eq!(body["code"], 10001);
    }

    /// 豆包引擎：未配 DOUBAO_COOKIE → 400 引导（不连网，配置检查先行）
    #[tokio::test]
    async fn speak_doubao_without_cookie_is_400() {
        let app = Router::new().merge(router()).with_state(test_state());
        let request = Request::builder()
            .method("POST")
            .uri("/api/tts/speak")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "voice": "taozi", "text": "你好", "engine": "doubao" }).to_string(),
            ))
            .expect("请求");
        let response = app.oneshot(request).await.expect("响应");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("响应体");
        let body: Value = serde_json::from_slice(&bytes).expect("JSON");
        assert_eq!(body["code"], 10001);
        assert!(
            body["message"]
                .as_str()
                .is_some_and(|m| m.contains("DOUBAO_COOKIE"))
        );
    }

    /// 豆包引擎：空文本 → 400（文本校验先行，不发起网络连接）
    #[tokio::test]
    async fn speak_doubao_empty_text_is_400() {
        let app = Router::new()
            .merge(router())
            .with_state(test_state_with_cookie());
        let request = Request::builder()
            .method("POST")
            .uri("/api/tts/speak")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "voice": "taozi", "text": "  ", "engine": "doubao" }).to_string(),
            ))
            .expect("请求");
        let response = app.oneshot(request).await.expect("响应");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("响应体");
        let body: Value = serde_json::from_slice(&bytes).expect("JSON");
        assert_eq!(body["code"], 10001);
    }
}
