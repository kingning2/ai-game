use axum::{Router, routing::get};

use state::AppState;

/// 基础设施探活端点，保持裸响应（不套 ApiResponse）
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "system",
    operation_id = "health",
    responses((status = 200, description = "探活成功，固定返回 ok", body = String))
)]
pub async fn health() -> &'static str {
    "ok"
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/health", get(health))
}
