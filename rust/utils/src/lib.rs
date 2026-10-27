#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! 跨包通用小工具。保持零业务语义，只放「任何 crate 都可能用」的基础函数。
//! 一个工具一个文件，这里统一 re-export；各工具的说明见对应模块与 README.md。

mod env;
mod fallible;

pub use env::{ConfigError, get_env, get_env_opt, get_env_parse};
pub use fallible::FallibleExt;
