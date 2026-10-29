//! 选手级模型配置端点：内置 12 名选手 + 自建选手各自的 provider / 模型 / Key。
//!
//! - `GET /api/roster`：全员有效配置（覆盖合并进全局后的形态，Key 脱敏）+ 各自覆盖明细
//! - `PUT /api/roster/{name}`：设置覆盖（字段缺省 = 继承全局；全字段缺省 = 清除覆盖回全局）；
//!   名单外选手视为自建选手直接入库
//! - `DELETE /api/roster/{name}`：删除自建选手（内置选手 400）
//!
//! 覆盖持久化到 `roster.json`；编排层用 `state.roster.resolve(name, global)` 取选手有效配置。
//! 前端 DTO 在 `apps/web` 手写对齐（字段变更时同步修改）。

use crate::error::{ApiError, ApiResponse, ApiResult, AppError};
use axum::{Json, Router, extract::Path as AxumPath, extract::State, routing::get};
use serde::{Deserialize, Serialize};
use state::{AppState, PlayerOverride};

use agent::ProviderKind;

use super::agent_config::AgentConfigDto;

/// 单个选手（GET 响应元素）：覆盖明细 + 合并后的有效配置
#[derive(Serialize, utoipa::ToSchema)]
pub struct RosterPlayerDto {
    /// 选手名（内置 12 人与前端 AGENT_POOL 对齐；自建选手为用户输入）
    pub name: String,
    /// 自定义头像（emoji 或 URL）；None = 前端用静态资源/首字兑底
    pub avatar: Option<String>,
    /// 是否存在有效覆盖（前端渲染“已单独配置”徽标）
    pub has_override: bool,
    /// 覆盖明细；None = 完全跟随全局
    pub player_override: Option<PlayerOverrideDto>,
    /// 有效配置（覆盖合并进全局后；Key 已脱敏）
    pub effective: AgentConfigDto,
}

/// 覆盖明细（GET 响应 / PUT 请求体双形态）：字段缺省 = 继承全局
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PlayerOverrideDto {
    /// 切换供应商（`openai` | `anthropic`）；缺省 = 继承全局
    pub provider: Option<String>,
    pub openai: Option<SettingsPatchDto>,
    pub anthropic: Option<SettingsPatchDto>,
    /// 单次回复 token 上限；缺省 = 继承全局
    pub max_tokens: Option<u32>,
    /// 思考模式（`enabled` | `disabled`）；缺省 = 继承全局（不发参数）
    pub thinking: Option<String>,
    /// 自定义头像（emoji 或图片 URL）；缺省 = 无自定义
    pub avatar: Option<String>,
}

/// 单协议连接参数覆盖；字段缺省 = 继承全局。
/// GET 响应中 `api_key` 恒为 null（脱敏到 `api_key_masked`），PUT 才接受真值
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SettingsPatchDto {
    pub base_url: Option<String>,
    pub model: Option<String>,
    /// API Key；缺省 = 继承全局，空串 = 明确清空。仅 PUT 接受；GET 恒为 null
    pub api_key: Option<String>,
    /// 脱敏 Key（保留末 4 位）；仅 GET 返回，PUT 忽略
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_masked: Option<String>,
}

/// GET /api/roster
#[utoipa::path(
    get,
    path = "/api/roster",
    tag = "roster",
    operation_id = "get_roster",
    responses(
        (status = 200, description = "全员选手配置（Key 已脱敏）", body = ApiResponse<Vec<RosterPlayerDto>>)
    )
)]
pub async fn get_roster(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<RosterPlayerDto>>>> {
    let players = state
        .roster
        .player_names()
        .iter()
        .map(|name| player_dto(&state, name))
        .collect();
    Ok(ApiResponse::ok(players))
}

/// PUT /api/roster/{name}
#[utoipa::path(
    put,
    path = "/api/roster/{name}",
    tag = "roster",
    operation_id = "put_roster_player",
    request_body = PlayerOverrideDto,
    params(("name" = String, Path, description = "选手名")),
    responses(
        (status = 200, description = "覆盖已落盘，返回该选手最新配置（全字段缺省 = 清除覆盖）", body = ApiResponse<RosterPlayerDto>),
        (status = 400, description = "字段非法（provider 未知名 / base_url 协议头缺失 / max_tokens 超界等）"),
        (status = 500, description = "配置文件落盘失败")
    )
)]
pub async fn put_roster_player(
    State(state): State<AppState>,
    AxumPath(name): AxumPath<String>,
    Json(update): Json<PlayerOverrideDto>,
) -> ApiResult<Json<ApiResponse<RosterPlayerDto>>> {
    if name.trim().is_empty() {
        return Err(AppError::BadRequest("选手名不能为空".to_string()).into());
    }
    let player_override = into_override(update)?;
    state.roster.set_override(&name, player_override)?;
    Ok(ApiResponse::ok(player_dto(&state, &name)))
}

/// DELETE /api/roster/{name}：删除自建选手（内置 12 人 400）
#[utoipa::path(
    delete,
    path = "/api/roster/{name}",
    tag = "roster",
    operation_id = "delete_roster_player",
    params(("name" = String, Path, description = "选手名")),
    responses(
        (status = 200, description = "自建选手已删除", body = ApiResponse<Vec<RosterPlayerDto>>),
        (status = 400, description = "内置选手不可删除")
    )
)]
pub async fn delete_roster_player(
    State(state): State<AppState>,
    AxumPath(name): AxumPath<String>,
) -> ApiResult<Json<ApiResponse<Vec<RosterPlayerDto>>>> {
    state.roster.remove_player(&name)?;
    let players = state
        .roster
        .player_names()
        .iter()
        .map(|name| player_dto(&state, name))
        .collect();
    Ok(ApiResponse::ok(players))
}

fn player_dto(state: &AppState, name: &str) -> RosterPlayerDto {
    let player_override = state.roster.override_of(name);
    let effective = state.roster.resolve(name, &state.agent.config());
    RosterPlayerDto {
        name: name.to_string(),
        avatar: player_override
            .as_ref()
            .and_then(|player_override| player_override.avatar.clone()),
        has_override: player_override.is_some(),
        player_override: player_override.map(override_dto),
        effective: AgentConfigDto::of(&effective),
    }
}

fn override_dto(player_override: PlayerOverride) -> PlayerOverrideDto {
    PlayerOverrideDto {
        provider: player_override.provider.map(|provider| {
            match provider {
                ProviderKind::OpenAi => "openai",
                ProviderKind::Anthropic => "anthropic",
            }
            .to_string()
        }),
        openai: player_override.openai.map(patch_dto),
        anthropic: player_override.anthropic.map(patch_dto),
        max_tokens: player_override.max_tokens,
        thinking: player_override.thinking.map(thinking_dto),
        avatar: player_override.avatar,
    }
}

fn thinking_dto(thinking: agent::ThinkingMode) -> String {
    match thinking {
        agent::ThinkingMode::Enabled => "enabled".to_string(),
        agent::ThinkingMode::Disabled => "disabled".to_string(),
    }
}

fn thinking_from(raw: &str) -> Result<agent::ThinkingMode, ApiError> {
    match raw {
        "enabled" => Ok(agent::ThinkingMode::Enabled),
        "disabled" => Ok(agent::ThinkingMode::Disabled),
        other => Err(AppError::BadRequest(format!(
            "thinking 无效：{other}（可选 enabled / disabled）"
        ))
        .into()),
    }
}

fn patch_dto(patch: state::SettingsPatch) -> SettingsPatchDto {
    SettingsPatchDto {
        base_url: patch.base_url,
        model: patch.model,
        // GET 不回明文 Key：脱敏到 api_key_masked，api_key 恒为 null
        api_key: None,
        api_key_masked: Some(crate::routes::agent_config::mask_key(
            &patch.api_key.unwrap_or_default(),
        )),
    }
}

fn into_override(update: PlayerOverrideDto) -> Result<Option<PlayerOverride>, ApiError> {
    let provider = match update.provider.as_deref() {
        None => None,
        Some("openai") => Some(ProviderKind::OpenAi),
        Some("anthropic") => Some(ProviderKind::Anthropic),
        Some(other) => {
            return Err(AppError::BadRequest(format!(
                "provider 无效：{other}（可选 openai / anthropic）"
            ))
            .into());
        }
    };
    let openai = update.openai.map(into_patch).transpose()?;
    let anthropic = update.anthropic.map(into_patch).transpose()?;
    if let Some(max_tokens) = update.max_tokens
        && (max_tokens == 0 || max_tokens > 1_000_000)
    {
        return Err(AppError::BadRequest("max_tokens 需在 1 ~ 1_000_000 之间".to_string()).into());
    }

    let player_override = PlayerOverride {
        provider,
        openai,
        anthropic,
        max_tokens: update.max_tokens,
        thinking: update.thinking.as_deref().map(thinking_from).transpose()?,
        avatar: update.avatar,
    };
    // 下发了对象但全空时 `set_override` 会自动当作清除（跟随全局）
    Ok(Some(player_override))
}

fn into_patch(patch: SettingsPatchDto) -> Result<state::SettingsPatch, ApiError> {
    if let Some(base_url) = &patch.base_url
        && (base_url.trim().is_empty()
            || (!base_url.starts_with("http://") && !base_url.starts_with("https://")))
    {
        return Err(AppError::BadRequest(format!(
            "base_url 需以 http:// 或 https:// 开头：{base_url}"
        ))
        .into());
    }
    if let Some(model) = &patch.model
        && model.trim().is_empty()
    {
        return Err(AppError::BadRequest("model 不能为空".to_string()).into());
    }
    Ok(state::SettingsPatch {
        base_url: patch.base_url,
        model: patch.model,
        api_key: patch.api_key,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/roster", get(get_roster))
        .route("/api/roster/{name}", axum::routing::put(put_roster_player))
        .route(
            "/api/roster/{name}",
            axum::routing::delete(delete_roster_player),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{Value, json};
    use state::{AgentState, RosterState};
    use tower::ServiceExt;

    fn app(state: state::AppState) -> axum::Router {
        Router::new().merge(router()).with_state(state)
    }

    fn test_state() -> state::AppState {
        // 内存态 roster：不碰真实配置目录，测试间零串扰
        state::AppState::new_with_roster(
            state::AppConfig::default(),
            AgentState::ephemeral(),
            RosterState::ephemeral(),
        )
    }

    async fn get_json(state: state::AppState, uri: &str) -> (StatusCode, Value) {
        let res = app(state)
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = res.status();
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    async fn put_json(state: state::AppState, uri: &str, body: Value) -> (StatusCode, Value) {
        let res = app(state)
            .oneshot(
                Request::put(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    async fn delete_json(state: state::AppState, uri: &str) -> (StatusCode, Value) {
        let res = app(state)
            .oneshot(Request::delete(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn get_roster_lists_all_players_with_effective_config() {
        let (status, value) = get_json(test_state(), "/api/roster").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(value["code"], 0);
        let players = value["data"].as_array().unwrap();
        assert_eq!(players.len(), 12);
        assert_eq!(players[0]["name"], "Claude Opus 5.5");
        // 无覆盖时 has_override=false，effective 继承全局默认配置
        assert_eq!(players[0]["has_override"], false);
        assert!(players[0]["player_override"].is_null());
        assert!(
            players[0]["effective"]["openai"]["base_url"]
                .as_str()
                .unwrap()
                .starts_with("http")
        );
    }

    #[tokio::test]
    async fn put_override_merges_and_masks_key() {
        let state = test_state();
        let (status, value) = put_json(
            state.clone(),
            "/api/roster/GPT-6.1",
            json!({"openai": {"model": "doubao-seed-1-6", "api_key": "sk-1234567890abcdef"}}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        assert_eq!(value["code"], 0);
        let data = &value["data"];
        assert_eq!(data["has_override"], true);
        // 有效配置：覆盖的 model/api_key 生效，其余继承全局
        assert_eq!(data["effective"]["openai"]["model"], "doubao-seed-1-6");
        assert_eq!(data["effective"]["openai"]["api_key_masked"], "****cdef");
        assert_eq!(data["effective"]["openai"]["has_key"], true);
        // 覆盖明细回显脱敏：api_key 恒 null，脱敏值在 api_key_masked
        assert_eq!(
            data["player_override"]["openai"]["model"],
            "doubao-seed-1-6"
        );
        assert!(
            data["player_override"]["openai"]["api_key"].is_null(),
            "GET 不得回明文 Key"
        );
        assert_eq!(
            data["player_override"]["openai"]["api_key_masked"],
            "****cdef"
        );
    }

    #[tokio::test]
    async fn put_thinking_roundtrip_and_validation() {
        let state = test_state();
        let (status, value) = put_json(
            state.clone(),
            "/api/roster/GPT-6.1",
            json!({"thinking": "enabled", "openai": {"model": "deepseek-v4-1-flash"}}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        assert_eq!(value["data"]["player_override"]["thinking"], "enabled");

        // 非法档位 400
        let (status, _) =
            put_json(state, "/api/roster/GPT-6.1", json!({"thinking": "turbo"})).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn put_empty_body_clears_override() {
        let state = test_state();
        let name = "/api/roster/Kimi%20K3.1%20Pro";
        let (_, value) = put_json(state.clone(), name, json!({"max_tokens": 4096})).await;
        assert_eq!(value["data"]["has_override"], true);

        let (_, value) = put_json(state, name, json!({})).await;
        assert_eq!(value["code"], 0);
        assert_eq!(value["data"]["has_override"], false);
        assert!(value["data"]["player_override"].is_null());
    }

    #[tokio::test]
    async fn custom_player_can_be_created_listed_and_deleted() {
        let state = test_state();
        // PUT 名单外选手 = 自建入库（带头像）
        let (status, value) = put_json(
            state.clone(),
            "/api/roster/%E7%A5%9E%E7%A7%98%E5%98%89%E5%AE%BE",
            json!({"avatar": "🎭", "openai": {"model": "doubao-seed-2-1-pro"}}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        assert_eq!(value["data"]["avatar"], "🎭");

        // GET 列表：内置 12 人 + 自建 1 人殿后
        let (_, value) = get_json(state.clone(), "/api/roster").await;
        let players = value["data"].as_array().unwrap();
        assert_eq!(players.len(), 13);
        assert_eq!(players[12]["name"], "神秘嘉宾");

        // 空名 400；内置选手删除 400
        let (status, _) = put_json(state.clone(), "/api/roster/%20%20", json!({})).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let (status, _) = delete_json(state.clone(), "/api/roster/GPT-6.1").await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        // 删除自建选手后列表回归 12 人
        let (_, value) =
            delete_json(state, "/api/roster/%E7%A5%9E%E7%A7%98%E5%98%89%E5%AE%BE").await;
        assert_eq!(value["code"], 0);
        assert_eq!(value["data"].as_array().unwrap().len(), 12);
    }

    #[tokio::test]
    async fn invalid_base_url_returns_400() {
        let (status, _) = put_json(
            test_state(),
            "/api/roster/Claude%20Opus%205.5",
            json!({"openai": {"base_url": "ftp://bad"}}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
