use std::env;
use std::env::VarError;
use std::str::FromStr;
use thiserror::Error;

/// 环境变量配置错误：区分「缺失」「非 Unicode」「解析失败」三种情况
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("读取不到环境变量 {name}，请检查是否设置了该环境变量")]
    NotPresent { name: String },

    #[error("环境变量 {name} 的值不是有效的 Unicode")]
    NotUnicode { name: String },

    #[error("环境变量 {name} 无法解析为 {target}")]
    Parse {
        name: String,
        /// 目标类型名（如 "u16"），来自 std::any::type_name
        target: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// 读取必需的环境变量：缺失 / 非 Unicode 都算错误
pub fn get_env(name: &str) -> Result<String, ConfigError> {
    match env::var(name) {
        Ok(value) => Ok(value),
        Err(VarError::NotPresent) => Err(ConfigError::NotPresent {
            name: name.to_string(),
        }),
        Err(VarError::NotUnicode(_)) => Err(ConfigError::NotUnicode {
            name: name.to_string(),
        }),
    }
}

/// 读取可选的环境变量：未设置返回 `Ok(None)`；设置了但非 Unicode 仍是错误
pub fn get_env_opt(name: &str) -> Result<Option<String>, ConfigError> {
    match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => Err(ConfigError::NotUnicode {
            name: name.to_string(),
        }),
    }
}

/// 读取必需的环境变量并解析为 `T`：缺失 → `NotPresent`，值不合法 → `Parse`（带底层 source 链）
///
/// 解析遵循 `T::from_str` 原始语义（不做 trim 等加工）。
///
/// ```no_run
/// use utils::get_env_parse;
///
/// let port: u16 = get_env_parse("SERVER_PORT").unwrap_or(17870);
/// ```
pub fn get_env_parse<T>(name: &str) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let raw = get_env(name)?;
    raw.parse::<T>().map_err(|source| ConfigError::Parse {
        name: name.to_string(),
        target: std::any::type_name::<T>(),
        source: Box::new(source),
    })
}

#[cfg(test)]
#[path = "env_tests.rs"]
mod tests;
