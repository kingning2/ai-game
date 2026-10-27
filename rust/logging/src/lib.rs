use tracing_subscriber::EnvFilter;

// 初始化全局日志系统
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .compact()
        .init();
}

pub use tracing::{Level, debug, error, info, span, trace, warn};
