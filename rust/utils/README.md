# utils

跨包通用小工具。保持零业务语义，只放"任何 crate 都可能用"的基础函数。

## 现有内容

### `get_env(name) -> Result<String, ConfigError>`

读取必需的环境变量。

### `get_env_opt(name) -> Result<Option<String>, ConfigError>`

读取可选的环境变量：未设置 → `Ok(None)`；设置了但值非 Unicode → `Err(NotUnicode)`。

### `get_env_parse<T: FromStr>(name) -> Result<T, ConfigError>`

读取 + 解析一体：缺失 → `NotPresent`；值不合法 → `Parse`（错误信息含目标类型名，source 链可取回底层 `ParseIntError` 等）。

```rust
use utils::get_env_parse;

let port: u16 = get_env_parse("SERVER_PORT").unwrap_or(17870);
```

### `ConfigError`（枚举，三种失败可区分）

| 变体               | 场景                 | 处理建议               |
| ------------------ | -------------------- | ---------------------- |
| `NotPresent`       | 变量未设置           | 有默认值就静默回退     |
| `NotUnicode`       | 值不是合法 Unicode   | 配置错误，报出来       |
| `Parse { source }` | 值无法解析为目标类型 | 有默认值就 warn 后回退 |

示例（server 的 main.rs 就是这么用的）：未设置静默用默认，设置了但格式错 warn 提示。

## 约定

- 一个工具一个文件（`src/<工具名>.rs`），`lib.rs` re-export
- 不依赖任何内部包、不引入重依赖
- env 读取不做 trim 等加工，解析遵循 `FromStr` 原始语义
