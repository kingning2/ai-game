//! 生命周期编排：启动 → 运行 → 优雅关闭
//!
//! `main()` 只调 [`run`]，三个阶段：
//!
//! 1. **bootstrap**（[`bootstrap`]）：日志初始化 → 配置加载 → 共享状态构建。
//!    当前无失败路径（env 回退保证 `AppConfig::from_env` 总能产出）；
//!    未来引入配置文件 / 连接池初始化时，改返回 `Result` 即可。
//! 2. **serve**（[`serve`]）：bind → 请求循环。`Ctrl+C` / 控制台关闭触发
//!    **优雅关闭**——停止接受新连接，等待进行中请求完成后才退出。
//!    注意：桌面壳退出时对 sidecar 是硬杀，优雅关闭主要覆盖开发模式（`pnpm server`
//!    按 Ctrl+C）与未来服务化场景。
//! 3. **shutdown 钩子**（[`on_shutdown`]）：无论优雅关闭还是异常退出都执行。
//!    当前输出最终统计；未来关连接池、flush 缓存、取消后台任务都挂这里。

use axum::{Router, middleware, response::IntoResponse};
use state::{AgentState, AppConfig, AppState};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::CorsLayer;

use crate::error::BoxError;
use crate::openapi as docs;
use crate::routes::rooms;
use crate::routes::{agent_config, health, info, leaderboard, roster, tts, werewolf};
use tracing::{error, info};

/// 生命周期入口：`main()` 唯一调用点
pub async fn run() {
    // —— 阶段 1：bootstrap ——
    let state = bootstrap();

    // —— 阶段 2：serve（bind 失败 / 运行异常都记日志，然后走关闭钩子）——
    if let Err(err) = serve(state.clone()).await {
        error!(error = %err, "服务异常退出");
    }

    // —— 阶段 3：shutdown 钩子（优雅关闭与异常退出都走这里）——
    on_shutdown(&state);
}

/// 全局日志初始化（自 logging crate 内联：单一调用方，不值得独立成包）
fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .compact()
        .init();
}

/// 阶段 1：初始化一切。当前无失败路径；需要可失败时改返回 `Result<AppState, BoxError>`
fn bootstrap() -> AppState {
    init_logging();

    info!("日志初始化成功");

    // 配置加载（env 读取/回退/警告逻辑）集中在 state::AppConfig::from_env
    let config = AppConfig::from_env();

    // agent 配置：由前端设置页下发，持久化到配置文件（不再读环境变量）
    let agent = match AgentState::default_path() {
        Some(path) => AgentState::load(path),
        None => AgentState::ephemeral(),
    };

    let state = AppState::new(config.clone(), agent);
    // 开机清扫：上次运行遗留的未完局标记 stale 结束（前端降级显示真实结算）
    if let Some(conn) = state
        .db
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
    {
        match db::finish_stale_games(conn) {
            Ok(n) if n > 0 => info!(swept = n, "已清扫上次运行遗留的未完局"),
            Ok(_) => {}
            Err(err) => error!(error = %err, "清扫未完局失败"),
        }
    }
    info!(address = %config.address, port = config.port, "配置加载完成");
    state
}

/// 阶段 2：bind + 请求循环 + 优雅关闭
async fn serve(state: AppState) -> Result<(), BoxError> {
    let listener =
        tokio::net::TcpListener::bind((state.config.address.as_str(), state.config.port)).await?;
    info!(
        address = %state.config.address,
        port = state.config.port,
        "服务就绪，等待请求"
    );

    axum::serve(listener, build_router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// 优雅关闭信号：`Ctrl+C` / 控制台关闭
/// （tokio 在 Windows 上同样捕获 CTRL_CLOSE / CTRL_SHUTDOWN 等控制事件）
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    info!("收到关闭信号：停止接受新请求，等待进行中请求完成");
}

/// 阶段 3：资源清理钩子。未来接数据库 / 后台任务时在这里释放
fn on_shutdown(state: &AppState) {
    info!(
        uptime_secs = state.uptime_secs(),
        total = state.stats.total(),
        active = state.stats.active(),
        ok = state.stats.ok(),
        client_errors = state.stats.client_errors(),
        server_errors = state.stats.server_errors(),
        "服务已退出"
    );
}

/// 路由组装：一个路由一个文件夹（src/routes/<name>/mod.rs），各自导出 router()，只在这里注册
fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .merge(info::router())
        .merge(agent_config::router())
        .merge(roster::router())
        .merge(rooms::router())
        .merge(leaderboard::router())
        .merge(werewolf::router())
        .merge(tts::router())
        .merge(docs::router())
        // 框架层错误统一转成 error 包的包裹响应（否则 axum 默认回裸文本，前端会归为 http 而非 business）：
        // 未匹配路径 → 404；路径存在但方法不对 → 405；handler panic → 500（原因进日志）
        .fallback(crate::error::fallback_404)
        .method_not_allowed_fallback(crate::error::fallback_405)
        .layer(CatchPanicLayer::custom(
            |panic: Box<dyn std::any::Any + Send>| {
                let detail = if let Some(s) = panic.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                error!(panic_detail = %detail, "handler 发生 panic");
                crate::error::ApiError(crate::error::AppError::internal(format!(
                    "handler panic: {detail}"
                )))
                .into_response()
            },
        ))
        .layer(CorsLayer::permissive())
        // 全局状态采集：from_fn_with_state 注入 AppState；track 放最外层（最后 .layer 的在外面），
        // panic 兑底(500)与 404 fallback 也被统计
        .layer(middleware::from_fn_with_state(state.clone(), state::track))
        // 共享状态最后注入；路由内用 State(state): State<AppState> 提取
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::build_router;
    use state::{AgentState, AppConfig, AppState};

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn routes_wired() {
        let app = build_router(AppState::new(AppConfig::default(), AgentState::ephemeral()));
        let get = |uri: &str| Request::builder().uri(uri).body(Body::empty()).unwrap();

        // health：裸 200
        let res = app.clone().oneshot(get("/api/health")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // info：包裹 200
        let res = app.clone().oneshot(get("/api/info")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // agent config：包裹 200（默认配置，Key 未配）
        let res = app.clone().oneshot(get("/api/agent/config")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // openapi 文档：JSON 200，含 agent 端点、不含文档端点自身
        let res = app
            .clone()
            .oneshot(get("/api-docs/openapi.json"))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), 1 << 22)
            .await
            .unwrap();
        let doc: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(doc["paths"]["/api/agent/config"].is_object());
        assert!(doc["paths"]["/api-docs/openapi.json"].is_null());

        // 未知路径 → 404 包裹
        let res = app.clone().oneshot(get("/api/nope")).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        // 方法不对 → 405 包裹
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
