//! ffmpeg crate 的单元测试（与 lib.rs 分离，对齐仓库 `*_tests.rs` 约定）。
//!
//! 进程编排链路依赖真实 ffmpeg 与时间，单元测试只锁纯函数行为：
//! `safe_name` 是文件名写入前唯一的防线（防路径穿越与非法字符）。

use super::*;

#[test]
fn safe_name_passes_clean_names() {
    assert_eq!(safe_name("room-123"), "room-123");
    assert_eq!(safe_name("my_room_2"), "my_room_2");
}

#[test]
fn safe_name_strips_traversal_and_illegal_chars() {
    assert_eq!(safe_name("../etc/passwd"), "---etc-passwd");
    assert_eq!(safe_name("..\\..\\win"), "------win");
    // 房、间、空格 = 3 个非法字符
    assert_eq!(safe_name("房间 1"), "---1");
    assert_eq!(safe_name("a/b\\c:d"), "a-b-c-d");
}

#[test]
fn safe_name_truncates_to_48_chars() {
    let long = "x".repeat(100);
    assert_eq!(safe_name(&long).len(), 48);
}

#[test]
fn unix_now_is_sane() {
    // 2020-01-01 之后、2100 年之前
    let now = unix_now();
    assert!(now > 1_577_836_800 && now < 4_102_444_800, "now={now}");
}
