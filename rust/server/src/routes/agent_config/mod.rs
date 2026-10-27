//! agent 配置端点：前端设置页的读写通道。
//!
//! - `GET /api/agent/config`：当前配置，**Key 脱敏后返回（永不回明文）**
//! - `PUT /api/agent/config`：更新配置（`api_key` 缺省 = 保留现值），
//!   成功后热生效并原子落盘到配置文件
//!
//! 前端 DTO 在 `apps/web` 手写对齐（字段变更时同步修改）。

use crate::error::{ApiError, ApiResponse, ApiResult, AppError};
use axum::{Json, Router, extract::State, routing::get};
use serde::{Deserialize, Serialize};
use state::AppState;

use agent::{AgentConfig, ProviderKind, ProviderSettings};

/// 当前 agent 配置（GET 响应；Key 只出脱敏形态）
#[derive(Serialize, utoipa::ToSchema)]
pub struct AgentConfigDto {
    /// 当前生效供应商：`openai` | `anthropic`
    pub provider: String,
    /// OpenAI 兼容协议连接参数（豆包兼容模式 / DeepSeek / Kimi / OpenRouter 等）
    pub openai: ProviderSettingsDto,
    /// Anthropic Messages 协议连接参数
    pub anthropic: ProviderSettingsDto,
    /// 单次回复 token 上限（Anthropic 必填；OpenAI 透传 max_tokens）
    pub max_tokens: u32,
}

/// 单协议连接参数（GET 响应形态）
#[derive(Serialize, utoipa::ToSchema)]
pub struct ProviderSettingsDto {
    /// API 根地址（不含尾斜杠）
    pub base_url: String,
    /// 默认模型
    pub model: String,
    /// 脱敏后的 Key（保留末 4 位，如 `****abcd`）；未配置为空串
    pub api_key_masked: String,
    /// 是否已配置 Key（前端据此渲染"已配置"状态，而不是让用户重填）
    pub has_key: bool,
}

/// PUT 请求体：全字段可选，缺省 = 保留现值
#[derive(Deserialize, utoipa::ToSchema)]
pub struct AgentConfigUpdate {
    /// 切换供应商（`openai` | `anthropic`）；缺省保留
    pub provider: Option<String>,
    pub openai: Option<ProviderSettingsUpdate>,
    pub anthropic: Option<ProviderSettingsUpdate>,
    /// 单次回复 token 上限；缺省保留
    pub max_tokens: Option<u32>,
}

/// 单协议参数更新。`api_key` 语义：缺省 = 保留现有 Key，空串 = 清除，非空 = 设置
#[derive(Deserialize, utoipa::ToSchema)]
pub struct ProviderSettingsUpdate {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
}

/// GET /api/agent/config
#[utoipa::path(
    get,
    path = "/api/agent/config",
    tag = "agent",
    operation_id = "get_agent_config",
    responses(
        (status = 200, description = "当前配置（Key 已脱敏）", body = ApiResponse<AgentConfigDto>)
    )
)]
pub async fn get_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<AgentConfigDto>>> {
    Ok(ApiResponse::ok(AgentConfigDto::of(&state.agent.config())))
}

/// PUT /api/agent/config
#[utoipa::path(
    put,
    path = "/api/agent/config",
    tag = "agent",
    operation_id = "update_agent_config",
    request_body = AgentConfigUpdate,
    responses(
        (status = 200, description = "更新成功，返回脱敏后的新配置（已落盘并热生效）", body = ApiResponse<AgentConfigDto>),
        (status = 400, description = "字段非法（provider 未知名 / base_url 协议头缺失 / max_tokens 超界等）"),
        (status = 500, description = "provider 构建或配置文件落盘失败")
    )
)]
pub async fn put_config(
    State(state): State<AppState>,
    Json(update): Json<AgentConfigUpdate>,
) -> ApiResult<Json<ApiResponse<AgentConfigDto>>> {
    let next = apply_update(state.agent.config(), update)?;
    state.agent.update(next)?;
    Ok(ApiResponse::ok(AgentConfigDto::of(&state.agent.config())))
}

/// 应用更新：以当前配置为底，逐字段覆盖（缺省保留），最后整体校验
fn apply_update(current: AgentConfig, update: AgentConfigUpdate) -> Result<AgentConfig, ApiError> {
    let mut next = current;

    if let Some(provider) = update.provider {
        next.provider = parse_provider(&provider)?;
    }
    if let Some(openai) = update.openai {
        apply_settings(&mut next.openai, openai);
    }
    if let Some(anthropic) = update.anthropic {
        apply_settings(&mut next.anthropic, anthropic);
    }
    if let Some(max_tokens) = update.max_tokens {
        next.max_tokens = max_tokens;
    }

    validate(&next)?;
    Ok(next)
}

fn apply_settings(target: &mut ProviderSettings, update: ProviderSettingsUpdate) {
    if let Some(base_url) = update.base_url {
        target.base_url = base_url;
    }
    if let Some(api_key) = update.api_key {
        target.api_key = api_key;
    }
    if let Some(model) = update.model {
        target.model = model;
    }
}

fn parse_provider(raw: &str) -> Result<ProviderKind, ApiError> {
    match raw {
        "openai" => Ok(ProviderKind::OpenAi),
        "anthropic" => Ok(ProviderKind::Anthropic),
        other => Err(AppError::BadRequest(format!(
            "provider 无效：{other}（可选 openai / anthropic）"
        ))
        .into()),
    }
}

/// 整体校验：更新坏一行配置就整体拒绝（而不是落盘后调用时才在对话报错）
fn validate(config: &AgentConfig) -> Result<(), ApiError> {
    for (name, settings) in [("openai", &config.openai), ("anthropic", &config.anthropic)] {
        if settings.base_url.trim().is_empty() {
            return Err(AppError::BadRequest(format!("{name}.base_url 不能为空")).into());
        }
        if !settings.base_url.starts_with("http://") && !settings.base_url.starts_with("https://") {
            return Err(AppError::BadRequest(format!(
                "{name}.base_url 需以 http:// 或 https:// 开头"
            ))
            .into());
        }
        if settings.model.trim().is_empty() {
            return Err(AppError::BadRequest(format!("{name}.model 不能为空")).into());
        }
    }

    if config.max_tokens == 0 || config.max_tokens > 1_000_000 {
        return Err(AppError::BadRequest("max_tokens 需在 1 ~ 1_000_000 之间".to_string()).into());
    }
    Ok(())
}

/// Key 脱敏：只保留末 4 位；短 Key（≤8 位）一律 `****`
pub(crate) fn mask_key(api_key: &str) -> String {
    let chars: Vec<char> = api_key.chars().collect();
    match chars.len() {
        0 => String::new(),
        1..=8 => "****".to_string(),
        _ => format!(
            "****{}",
            chars[chars.len() - 4..].iter().collect::<String>()
        ),
    }
}

impl AgentConfigDto {
    pub(crate) fn of(config: &AgentConfig) -> Self {
        Self {
            provider: match config.provider {
                ProviderKind::OpenAi => "openai",
                ProviderKind::Anthropic => "anthropic",
            }
            .to_string(),
            openai: ProviderSettingsDto::of(&config.openai),
            anthropic: ProviderSettingsDto::of(&config.anthropic),
            max_tokens: config.max_tokens,
        }
    }
}

impl ProviderSettingsDto {
    pub(crate) fn of(settings: &ProviderSettings) -> Self {
        Self {
            base_url: settings.base_url.clone(),
            model: settings.model.clone(),
            api_key_masked: mask_key(&settings.api_key),
            has_key: !settings.api_key.is_empty(),
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/agent/config", get(get_config).put(put_config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{Value, json};
    use state::AgentState;
    use tower::ServiceExt;

    fn app(state: state::AppState) -> axum::Router {
        Router::new().merge(router()).with_state(state)
    }

    fn test_state() -> state::AppState {
        AppState::new(state::AppConfig::default(), AgentState::ephemeral())
    }

    #[tokio::test]
    async fn get_config_masks_key() {
        let state = test_state();
        // 预置一个 Key，验证 GET 只出脱敏形态
        let mut config = agent::AgentConfig::default();
        config.openai.api_key = "sk-1234567890abcdef".to_string();
        state.agent.update(config).unwrap();

        let response = app(state)
            .oneshot(
                Request::builder()
                    .uri("/api/agent/config")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1 << 20)
            .await
            .unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["code"], 0);
        let masked = value["data"]["openai"]["api_key_masked"].as_str().unwrap();
        assert_eq!(masked, "****cdef");
        assert_eq!(value["data"]["openai"]["has_key"], true);
        // 明文绝不出现在响应里
        assert!(!String::from_utf8_lossy(&body).contains("abcdef"));
    }

    #[tokio::test]
    async fn put_config_updates_and_keeps_key_on_absent() {
        // 先配 Key
        let mut config = agent::AgentConfig::default();
        config.openai.api_key = "sk-keep-me-secret".to_string();
        let agent_state = AgentState::ephemeral();
        agent_state.update(config).unwrap();
        let state = AppState::new(state::AppConfig::default(), agent_state);

        // 只改 model，不带 api_key → Key 保留
        let response = app(state.clone())
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/agent/config")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"openai": {"model": "gpt-4o"}}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let current = state.agent.config();
        assert_eq!(current.openai.model, "gpt-4o");
        assert_eq!(current.openai.api_key, "sk-keep-me-secret");
    }

    #[tokio::test]
    async fn put_config_rejects_bad_base_url() {
        let response = app(test_state())
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/agent/config")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"openai": {"base_url": "not-a-url"}}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn mask_key_rules() {
        assert_eq!(mask_key(""), "");
        assert_eq!(mask_key("short"), "****");
        assert_eq!(mask_key("sk-1234567890abcdef"), "****cdef");
    }

    #[test]
    fn validate_accepts_default_config() {
        assert!(validate(&AgentConfig::default()).is_ok());
    }
}
