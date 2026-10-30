//! trybuild：宏的编译成功/失败场景。
//!
//! 失败用例用 .stderr 快照锁定错误文案（错误指向原始类型/字段的 span）。
#![cfg_attr(test, allow(clippy::unwrap_used))]

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/pass_*.rs");
    t.compile_fail("tests/trybuild/fail_*.rs");
}
