# server

axum 接口实现，总编排其他包（error / logging / utils）。默认监听 `127.0.0.1:17870`，是 web 前端与桌面 sidecar 共用的后端。

## 启动

```bash
pnpm server          # 单跑（根目录）
pnpm server:watch    # 热更新：rust/ 下任何变化自动重编译重启（watchexec）
```

⚠️ 与 `pnpm dev:win`（桌面壳）同时跑会端口冲突，二选一。

## 环境变量

配置加载统一在 `state::AppConfig::from_env()`（缺失/非法的回退与告警规则见 `rust/state/README.md`）：

| 变量          | 默认        | 说明                                     |
| ------------- | ----------- | ---------------------------------------- |
| `SERVER_NAME` | `127.0.0.1` | 绑定地址                                 |
| `SERVER_PORT` | `17870`     | 绑定端口（前端约定值，改了前端就连不上） |

## 路由约定（一文件夹一路由）

- 位置：`src/routes/<name>/mod.rs`，各自导出 `pub fn router() -> Router<AppState>`（签名统一，是否用 State 由 handler 自定）
- 注册：只允许在 `main.rs` 的 `Router::new().merge(...)` 里挂载，`.with_state(state)` 在所有 layer 之后统一注入
- 现有端点：
  - `GET /api/health` — 探活，**裸响应**（不套包裹），sidecar 就绪探测等基础设施用
  - `GET /api/info` — 服务元信息（版本/地址/端口/运行秒数/实时请求统计），标准包裹响应，State 提取的示范写法

## handler 写法约定

- 返回值统一 `ApiResult<Json<ApiResponse<T>>>`
- 提取器用 `Result<Query<T>, QueryRejection>` 接收，`?` 自动转 400 包裹（不要裸 `Query<T>`，那会走 axum 默认裸文本）
- 业务错误 `Err(AppError::Xxx.into())`，不手写 `code`/`status` 映射

## 框架层错误与状态采集（main.rs，勿动顺序）

内→外：`fallback`（404）→ `method_not_allowed_fallback`（405）→ `CatchPanicLayer`（500）→ `CorsLayer` → `track`（全局统计，最外层）→ `with_state`。前四者全部转成 error 包的包裹响应；track 在最外层保证兑底响应也被统计。

## 前后端 DTO

Rust 侧结构体定义在路由模块（如 `routes/info/mod.rs` 的 `ServerInfo`），前端在消费处手写同名 interface 对齐，字段变更时两侧同步修改，详见根 README「前后端 DTO 约定」。
