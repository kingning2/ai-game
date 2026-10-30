---
name: coding-guidelines
description: 'Use when asking about Rust code style or best practices, including test organization. Keywords: naming, formatting, comment, clippy, rustfmt, lint, code style, best practice, P.NAM, G.FMT, code review, naming convention, variable naming, function naming, type naming, unit test, test organization, cfg(test), integration test, 测试规范, 测试组织, 单元测试, 集成测试, 测试文件, 命名规范, 代码风格, 格式化, 最佳实践, 代码审查, 怎么命名'
source: https://rust-coding-guidelines.github.io/rust-coding-guidelines-zh/
user-invocable: false
---

# Rust Coding Guidelines (50 Core Rules)

## Naming (Rust-Specific)

| Rule                | Guideline                                               |
| ------------------- | ------------------------------------------------------- |
| No `get_` prefix    | `fn name()` not `fn get_name()`                         |
| Iterator convention | `iter()` / `iter_mut()` / `into_iter()`                 |
| Conversion naming   | `as_` (cheap &), `to_` (expensive), `into_` (ownership) |
| Static var prefix   | `G_CONFIG` for `static`, no prefix for `const`          |

## Data Types

| Rule                  | Guideline                                         |
| --------------------- | ------------------------------------------------- |
| Use newtypes          | `struct Email(String)` for domain semantics       |
| Prefer slice patterns | `if let [first, .., last] = slice`                |
| Pre-allocate          | `Vec::with_capacity()`, `String::with_capacity()` |
| Avoid Vec abuse       | Use arrays for fixed sizes                        |

## Strings

| Rule                   | Guideline                               |
| ---------------------- | --------------------------------------- |
| Prefer bytes           | `s.bytes()` over `s.chars()` when ASCII |
| Use `Cow<str>`         | When might modify borrowed data         |
| Use `format!`          | Over string concatenation with `+`      |
| Avoid nested iteration | `contains()` on string is O(n*m)        |

## Error Handling

| Rule                       | Guideline                   |
| -------------------------- | --------------------------- |
| Use `?` propagation        | Not `try!()` macro          |
| `expect()` over `unwrap()` | When value guaranteed       |
| Assertions for invariants  | `assert!` at function entry |

## Memory

| Rule                         | Guideline                    |
| ---------------------------- | ---------------------------- |
| Meaningful lifetimes         | `'src`, `'ctx` not just `'a` |
| `try_borrow()` for RefCell   | Avoid panic                  |
| Shadowing for transformation | `let x = x.parse()?`         |

## Concurrency

| Rule                          | Guideline                      |
| ----------------------------- | ------------------------------ |
| Identify lock ordering        | Prevent deadlocks              |
| Atomics for primitives        | Not Mutex for bool/usize       |
| Choose memory order carefully | Relaxed/Acquire/Release/SeqCst |

## Async

| Rule                          | Guideline         |
| ----------------------------- | ----------------- |
| Sync for CPU-bound            | Async is for I/O  |
| Don't hold locks across await | Use scoped guards |

## Macros

| Rule                   | Guideline                         |
| ---------------------- | --------------------------------- |
| Avoid unless necessary | Prefer functions/generics         |
| Follow Rust syntax     | Macro input should look like Rust |

## Deprecated → Better

| Deprecated              | Better                | Since |
| ----------------------- | --------------------- | ----- |
| `lazy_static!`          | `std::sync::OnceLock` | 1.70  |
| `once_cell::Lazy`       | `std::sync::LazyLock` | 1.80  |
| `std::sync::mpsc`       | `crossbeam::channel`  | -     |
| `std::sync::Mutex`      | `parking_lot::Mutex`  | -     |
| `failure`/`error-chain` | `thiserror`/`anyhow`  | -     |
| `try!()`                | `?` operator          | 2018  |

## Testing（强制，完整规范见 [references/testing.md](references/testing.md)）

| Rule | Guideline |
| --------------------- | -------------------------------------------------- |
| 测试与业务分离 | 默认不内联 `#[cfg(test)] mod tests`，放独立 `tests.rs` |
| 单元测试归属模块 | 模块目录下 `tests.rs`（`#[cfg(test)] #[path = "tests.rs"] mod tests;`） |
| 集成测试归属 crate | crate 根 `tests/`，fixtures 放 `tests/fixtures/` |
| 私有实现可例外 | 需要访问私有函数时才用内联测试模块，禁止为测试改 `pub` |
| 围绕行为组织 | 按模块职责分文件，不按源文件数量机械配套，不堆一个文件 |
| 质量 > 数量 | 覆盖正常/边界/异常/错误分支；Mock 外部依赖，禁连生产环境 |
| Bug 修复配回归测试 | 修 Bug 必须先加能复现的回归测试，禁止删弱测试规避失败 |
| 交付必跑 | `cargo test` 相关模块 → 全量；条件允许 `cargo fmt --check` + `cargo clippy`；报告测试命令与结果 |

## Quick Reference

```
Naming: snake_case (fn/var), CamelCase (type), SCREAMING_CASE (const)
Format: rustfmt (just use it)
Docs: /// for public items, //! for module docs
Lint: #![warn(clippy::all)]
```

Claude knows Rust conventions well. These are the non-obvious Rust-specific rules.
