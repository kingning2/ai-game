# state

全局状态管理中心：`AppState` 是 axum 共享状态的唯一容器。静态配置、运行时统计、未来的连接池/缓存/会话表都挂这里。

## 结构

```
config.rs      AppConfig::from_env()   静态配置（env 加载唯一入口）
stats.rs       Stats                   全局请求统计（无锁原子计数）
track.rs       track                   axum 中间件：请求进出自动采集统计
app_state.rs   AppState                共享状态容器（config + stats + started_at）
```

## AppState 约定

- handler 提取：`State(state): State<AppState>`；路由签名统一 `Router<AppState>`
- `with_state(state)` 在 main.rs 所有 layer 之后统一注入
- 只放 `Clone + Send + Sync` 的东西（axum 要求 state 可克隆共享）
- 共享可变状态用原子类型或 `Arc<Mutex/RwLock<...>>`，**锁封在字段私有 + 方法访问后面**，不把锁类型暴露给 handler
- 扩展模式：加字段 →（如需自动采集）写访问方法 → 经 `/api/info` 或新端点暴露

## Stats / track 中间件

- track 必须**放最外层**（main.rs 里 `.layer()` 顺序在 CorsLayer 之后），
  这样 panic 兜底（CatchPanicLayer 转出的 500）与 404 fallback 也被统计：
  `.layer(middleware::from_fn_with_state(state.clone(), state::track))`
- ⚠️ 带 `State` 提取的中间件必须用 `from_fn_with_state`，不能用 `from_fn`（后者硬编码 `S = ()`，编译报 Service trait 不满足）
- `Stats` 的 `Clone` 共享同一计数器（内部 `Arc`）——这正是全局状态语义
- `begin()/end()` 由 track 维护，业务代码不要直接调
- `/api/info` 响应里 `active >= 1` 是正常时序（info 自己的请求还在处理中）

## AppConfig 加载规则

| 变量          | 默认        | 非法值行为          |
| ------------- | ----------- | ------------------- |
| `SERVER_NAME` | `127.0.0.1` | 用默认              |
| `SERVER_PORT` | `17870`     | warn 提示后回退默认 |

- 未设置 → 静默用默认（正常启动路径，不打日志）
- 设置了但非法 → `tracing::warn!` 提示后回退（配置错误要让人看见）

## 测试约定

env 是进程级全局状态且 edition 2024 下 `set_var/remove_var` 是 unsafe：

- 多个场景共用同一变量时，**放同一个测试函数里顺序执行**（见 `config::tests::from_env_scenarios`），避免并行竞争
- 测试结束清理变量，不污染同进程其他测试
- 中间件测试用 `tower::ServiceExt::oneshot` 直接打 Router（见 `track::tests::tracks_all_requests`）
