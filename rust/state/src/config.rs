use utils::{ConfigError, get_env, get_env_opt, get_env_parse};

/// 服务配置：全部来自环境变量，缺失/非法时回退默认值
///
/// 加载规则（from_env 内聚，main 与测试都不要再手写 env 逻辑）：
/// - 未设置 → 静默用默认
/// - 设置了但非法 → warn 提示后回退默认
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    /// 绑定地址（`SERVER_NAME`，默认 `127.0.0.1`）
    pub address: String,
    /// 绑定端口（`SERVER_PORT`，默认 `17870`，前端/sidecar 约定值）
    pub port: u16,
    /// 豆包云端 TTS 的登录 Cookie（`DOUBAO_COOKIE`，可选；缺失时豆包引擎不可用）
    pub doubao_cookie: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 17870,
            doubao_cookie: None,
        }
    }
}

impl AppConfig {
    /// 从环境变量加载配置（唯一入口）
    pub fn from_env() -> Self {
        let mut config = Self::default();

        config.address = get_env("SERVER_NAME").unwrap_or(config.address);

        config.port = match get_env_parse("SERVER_PORT") {
            Ok(port) => port,
            Err(ConfigError::NotPresent { .. }) => config.port,
            Err(err) => {
                tracing::warn!(error = %err, "SERVER_PORT 无效，回退默认端口 17870");
                config.port
            }
        };

        // 可选：豆包 Cookie（trim 后为空视为未配置）
        config.doubao_cookie = get_env_opt("DOUBAO_COOKIE")
            .ok()
            .flatten()
            .map(|cookie| cookie.trim().to_string())
            .filter(|cookie| !cookie.is_empty());

        config
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
