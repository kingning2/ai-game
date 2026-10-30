# Codex 复用的 lint 配置（来源：openai/codex codex-rs，Apache-2.0）

> 抓取自 2025 年 main 分支。workspace 根 `Cargo.toml` 直接抄第 1 段；`clippy.toml` 抄第 2 段模板（已剔除 Codex 业务特有的 ratatui/sqlx 禁用项）。
> 生效条件：每个 member crate 的 `Cargo.toml` 必须有 `[lints] workspace = true`（`rust-cargo-workspace-designer` 的 CW 规则也会检查这个）。

## 1. 根 Cargo.toml 追加（逐字可复制）

```toml
[workspace.lints.rust]
# 空：按需加，例如 unsafe_code = "forbid"

[workspace.lints.clippy]
await_holding_invalid_type = "deny"
await_holding_lock = "deny"
disallowed_methods = "deny"
expect_used = "deny"
identity_op = "deny"
manual_clamp = "deny"
manual_filter = "deny"
manual_find = "deny"
manual_flatten = "deny"
manual_map = "deny"
manual_memcpy = "deny"
manual_non_exhaustive = "deny"
manual_ok_or = "deny"
manual_range_contains = "deny"
manual_retain = "deny"
manual_strip = "deny"
manual_try_fold = "deny"
manual_unwrap_or = "deny"
needless_borrow = "deny"
needless_borrowed_reference = "deny"
needless_collect = "deny"
needless_late_init = "deny"
needless_option_as_deref = "deny"
needless_question_mark = "deny"
needless_update = "deny"
redundant_clone = "deny"
redundant_closure = "deny"
redundant_closure_for_method_calls = "deny"
redundant_static_lifetimes = "deny"
trivially_copy_pass_by_ref = "deny"
uninlined_format_args = "deny"
unnecessary_filter_map = "deny"
unnecessary_lazy_evaluations = "deny"
unnecessary_sort_by = "deny"
unnecessary_to_owned = "deny"
unwrap_used = "deny"
```

这批 lint 分三类语义：

- **防蠢优化**：`redundant_clone`、`needless_collect`、`manual_memcpy`、`unnecessary_to_owned`
- **防偷懒逃逸**：`unwrap_used`、`expect_used`（生产代码禁 panic，测试放行靠 clippy.toml）
- **强制惯用写法**：`manual_*` 系列、`uninlined_format_args`、`needless_question_mark`

## 2. workspace 根 clippy.toml 模板

```toml
# 测试代码放行 unwrap/expect（生产代码仍 deny）
allow-expect-in-tests = true
allow-unwrap-in-tests = true

# 持锁跨 await 的类型拉黑（有 tokio 就保留这三行）
await-holding-invalid-types = [
  "tokio::sync::MutexGuard",
  "tokio::sync::RwLockReadGuard",
  "tokio::sync::RwLockWriteGuard",
]

# 项目级方法黑名单：按本项目需要追加，格式：
# { path = "crate::module::func", reason = "为什么禁用、用什么替代" }
disallowed-methods = []

# 允许更大的错误类型变体（Err 变体 >128 字节不再报 result_large_err）
large-error-threshold = 256
```

## 3. 注意事项

- `disallowed_methods = "deny"` 但 `disallowed-methods` 列表为空时 lint 不报任何东西——它只是给项目留扩展点，Codex 用它禁止绕过 SQLite shim。有价值就加自己的条目（如禁 `std::process::Command::new` 直调、强制走统一封装）。
- `unwrap_used/expect_used = deny` 很严：新代码所有 `unwrap()` 都会被拦。如果存量代码大量 unwrap，先用 `"warn"` 起步，清完再升 deny。
- `trivially_copy_pass_by_ref` 需要较新 clippy 且对 FFI 风格代码可能误报，误报时局部 `#[allow]` + 注释原因。
- 该表不含 `all = "warn"` 组，建议配合 skill 主文档根清单里的 `[workspace.lints.clippy] all = { level = "warn", priority = -1 }` 一起用（Codex 没用是因为它 CI 另有检查）。
