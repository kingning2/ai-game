//! config.rs 的单元测试（由内联 mod tests 迁入）。

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
