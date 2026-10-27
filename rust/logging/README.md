# logging

应用整体日志：初始化全局 tracing subscriber + re-export 常用宏。

## 用法

```rust
use logging::{error, info};   // re-export 自 tracing

logging::init();              // main() 里最先调用，只调一次
info!("日志初始化成功");
error!(error = %err, "绑定服务端口失败");
```

## 行为

- 过滤规则：`RUST_LOG` 环境变量优先（标准 EnvFilter 语法，如 `RUST_LOG=debug`、`RUST_LOG=server=trace`），未设置时默认 `info`
- 输出：stdout，compact 格式，带 target（如 `error::api_error`）与线程 ID
- 与 error 包的分工：**日志分级策略在 error 包**（4xx=warn / 5xx=error），本包只负责初始化与宏转发

## 约定

- 除 `main()` 外不要在任何地方再调 `logging::init()`
- 字段化写法优先（`error!(error = %err, "...")`），不要全拼进消息字符串
