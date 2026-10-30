//! stats.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

#[test]
fn counts_by_status_class() {
    let stats = Stats::default();
    stats.begin();
    stats.begin();
    stats.end(StatusCode::OK);
    stats.end(StatusCode::BAD_REQUEST);

    assert_eq!(stats.total(), 2);
    assert_eq!(stats.active(), 0, "begin/end 配对后进行中归零");
    assert_eq!(stats.ok(), 1);
    assert_eq!(stats.client_errors(), 1);
    assert_eq!(stats.server_errors(), 0);
}

#[test]
fn clone_shares_counters() {
    let stats = Stats::default();
    let cloned = stats.clone();
    cloned.begin();
    assert_eq!(stats.total(), 1, "Clone 共享同一计数器（全局状态语义）");
}

#[test]
fn non_http_status_ignored_in_classify() {
    let stats = Stats::default();
    stats.begin();
    stats.end(StatusCode::SWITCHING_PROTOCOLS); // 1xx，不归类
    assert_eq!(
        (stats.ok(), stats.client_errors(), stats.server_errors()),
        (0, 0, 0)
    );
}
