use crate::error::{ApiResponse, ApiResult};
use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use state::AppState;

/// 服务元信息：前端可用于验证 sidecar 连通性、展示版本与运行时长
///
/// 前端 DTO 在 apps/web/src/App.tsx 手写对齐（字段变更时同步修改）
#[derive(Serialize, utoipa::ToSchema)]
pub struct ServerInfo {
    /// server crate 版本（编译期取自 CARGO_PKG_VERSION）
    pub version: String,
    /// 绑定地址（来自 AppConfig）
    pub address: String,
    /// 绑定端口
    pub port: u16,
    /// 服务已运行秒数
    pub uptime_secs: u64,
    /// 实时请求统计（track 中间件自动采集）
    pub stats: ServerStats,
}

/// 请求统计快照（字段与 state::Stats 的访问器一一对应）
#[derive(Serialize, utoipa::ToSchema)]
pub struct ServerStats {
    /// 自启动以来总请求数
    pub total: u64,
    /// 进行中请求数
    pub active: u64,
    /// 2xx 响应数
    pub ok: u64,
    /// 4xx 响应数
    pub client_errors: u64,
    /// 5xx 响应数
    pub server_errors: u64,
}

/// State 注入示范：`State(state): State<AppState>` 从共享状态取配置与启动时刻
#[utoipa::path(
    get,
    path = "/api/info",
    tag = "system",
    operation_id = "get_server_info",
    responses(
        (status = 200, description = "服务元信息与实时请求统计", body = ApiResponse<ServerInfo>)
    )
)]
pub async fn info(State(state): State<AppState>) -> ApiResult<Json<ApiResponse<ServerInfo>>> {
    Ok(ApiResponse::ok(ServerInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        address: state.config.address.clone(),
        port: state.config.port,
        uptime_secs: state.uptime_secs(),
        stats: ServerStats {
            total: state.stats.total(),
            active: state.stats.active(),
            ok: state.stats.ok(),
            client_errors: state.stats.client_errors(),
            server_errors: state.stats.server_errors(),
        },
    }))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/info", get(info))
}
