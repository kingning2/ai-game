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
mod tests {
    use super::*;
    use std::env;

    /// 所有碰 env 的场景放同一个测试里顺序执行，
    /// 避免 env 全局状态在并行测试间竞争
    #[test]
    fn from_env_scenarios() {
        // 1. 未设置 → 全默认（先清掉可能被并行用例残留的 DOUBAO_COOKIE）
        unsafe { env::remove_var("SERVER_NAME") };
        unsafe { env::remove_var("SERVER_PORT") };
        unsafe { env::remove_var("DOUBAO_COOKIE") };
        assert_eq!(AppConfig::from_env(), AppConfig::default());

        // 2. 合法覆盖
        unsafe { env::set_var("SERVER_NAME", "0.0.0.0") };
        unsafe { env::set_var("SERVER_PORT", "17871") };
        let config = AppConfig::from_env();
        assert_eq!((config.address.as_str(), config.port), ("0.0.0.0", 17871));

        // 3. 非法值 → warn 后回退默认
        unsafe { env::set_var("SERVER_PORT", "not-a-port") };
        let config = AppConfig::from_env();
        assert_eq!(config.port, 17870);
        assert_eq!(config.address, "0.0.0.0"); // 其他字段不受影响

        // 清理，避免污染同进程的其他测试
        unsafe { env::remove_var("SERVER_NAME") };
        unsafe { env::remove_var("SERVER_PORT") };

        // 4. DOUBAO_COOKIE：未设置/空白 → None；设置 → trim 后保留
        unsafe { env::set_var("DOUBAO_COOKIE", "   ") };
        assert_eq!(AppConfig::from_env().doubao_cookie, None);
        unsafe { env::set_var("DOUBAO_COOKIE", " sessionid=abc;  ") };
        assert_eq!(
            AppConfig::from_env().doubao_cookie.as_deref(),
            Some("sessionid=abc;")
        );

        // 清理，避免污染同进程的其他测试
        unsafe { env::remove_var("SERVER_NAME") };
        unsafe { env::remove_var("SERVER_PORT") };
        unsafe { env::remove_var("DOUBAO_COOKIE") };
    }
}
