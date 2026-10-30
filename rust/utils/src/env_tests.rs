//! env.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use std::env;

// edition 2024 下 set_var/remove_var 是 unsafe（进程级全局状态，多线程有竞争）；
// 每个测试用互不相同的变量名规避竞争
#[test]
fn missing_var() {
    let name = "UTILS_TEST_MISSING_XYZ";
    unsafe { env::remove_var(name) };
    assert!(matches!(
        get_env(name).unwrap_err(),
        ConfigError::NotPresent { .. }
    ));
    assert!(get_env_opt(name).unwrap().is_none());
}

#[test]
fn present_var() {
    let name = "UTILS_TEST_PRESENT_XYZ";
    unsafe { env::set_var(name, "hello") };
    assert_eq!(get_env(name).unwrap(), "hello");
    assert_eq!(get_env_opt(name).unwrap().as_deref(), Some("hello"));
}

#[test]
fn parse_ok() {
    let name = "UTILS_TEST_PARSE_OK_XYZ";
    unsafe { env::set_var(name, "17870") };
    let port: u16 = get_env_parse(name).unwrap();
    assert_eq!(port, 17870);
}

#[test]
fn parse_bad_value_has_source() {
    let name = "UTILS_TEST_PARSE_BAD_XYZ";
    unsafe { env::set_var(name, "not-a-port") };
    let err = get_env_parse::<u16>(name).unwrap_err();
    assert!(matches!(err, ConfigError::Parse { .. }));
    assert!(
        err.to_string().contains("u16"),
        "错误信息要提示目标类型：{err}"
    );
    assert!(
        std::error::Error::source(&err).is_some(),
        "Parse 要带 source 链"
    );
}

#[test]
fn parse_missing_is_not_present() {
    let name = "UTILS_TEST_PARSE_MISSING_XYZ";
    unsafe { env::remove_var(name) };
    assert!(matches!(
        get_env_parse::<u16>(name).unwrap_err(),
        ConfigError::NotPresent { .. }
    ));
}
