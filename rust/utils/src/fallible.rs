//! 无 panic 纪律的统一通道：可失败操作一律返回 [`Result`]，由上层领域错误构造器接管。
//!
//! 全仓约定（workspace clippy 门禁强制）：非测试代码禁止 `unwrap` / `expect` / `panic!`。
//! 底层错误（reqwest / serde_json / io …）需要转成语义错误时，统一走
//! [`FallibleExt::wrap_err`]——不要手写 `map_err(|e| X(e.to_string()))`，更不要 `expect`。

use std::fmt::Display;

/// 把底层错误统一转成领域错误的扩展 trait
///
/// ```rust
/// use utils::FallibleExt;
///
/// let parsed: Result<u32, std::num::ParseIntError> = "not a number".parse();
/// let code = parsed.wrap_err(|message| format!("端口解析失败：{message}"));
/// assert!(code.is_err());
/// ```
pub trait FallibleExt<T, E>: Sized {
    /// `Err` 时把底层错误 `to_string()` 后交给领域错误构造器。
    ///
    /// 元组变体构造器可直接传（如 `AgentError::Config`），需要拼上下文就传闭包。
    fn wrap_err<E2>(self, wrap: impl FnOnce(String) -> E2) -> Result<T, E2>;
}

impl<T, E: Display> FallibleExt<T, E> for Result<T, E> {
    fn wrap_err<E2>(self, wrap: impl FnOnce(String) -> E2) -> Result<T, E2> {
        self.map_err(|err| wrap(err.to_string()))
    }
}
