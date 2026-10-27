# agent-packages

Apple HIG 风格、令牌驱动的 React 组件库 monorepo（面向 AI Agent 控制台场景）。

> 📐 工程规范：**[AGENTS.md](AGENTS.md)**（总纲与索引）→ [rules/](rules/README.md)（分层规则：项目结构 / 编码规范 / UI 设计 / 组件指南）

pnpm monorepo。

```
├── apps/
│   ├── web         # 主应用：Vite 8 + React 19 + TS + Tailwind 4（主要开发都在这里）
│   └── desktop     # Tauri 2 壳：只有 src-tauri + sidecar 脚本，界面装载 web 的产物
├── packages/      # 发布包
│   ├── shared     # 通用工具库（请求层：原生 fetch，零第三方依赖）
│   └── shared-ui  # React 组件库（tsdown / rolldown 构建）
├── rust/          # Cargo workspace（多包）
│   └── server     # axum sidecar 服务，由桌面壳随应用拉起
├── .oxlintrc.json    # Oxlint 配置（lint，覆盖全仓库）
├── .oxfmtrc.json     # Oxfmt 配置（格式化，兼容 prettier 风格）
├── tsconfig.base.json # TS 基础配置，各包 extends
└── pnpm-workspace.yaml
```

## 技术栈

| 部分       | 工具                                                                       |
| ---------- | -------------------------------------------------------------------------- |
| 桌面壳     | Tauri 2（Rust + WebView2）                                                 |
| 后端服务   | axum，以 sidecar 随行进程运行，前端走 HTTP（不用 Tauri IPC）               |
| 前端构建   | Vite 8（内置 rolldown 打包器）                                             |
| 样式       | Tailwind CSS 4（CSS-first 配置，`@tailwindcss/vite` 插件，无 config 文件） |
| 组件库构建 | tsdown（rolldown，tsup 继任者），产物 `dist/index.mjs` + `.d.mts`          |

桌面端前置条件：Rust（`x86_64-pc-windows-msvc`）与 WebView2 Runtime。

## Rust 侧架构（rust/ workspace）

- `rust/Cargo.toml` 是唯一 Cargo workspace 根，共享 `rust/target`；`src-tauri` 通过 `workspace = "../../../rust"` 挂入（见其 Cargo.toml）
- `rust/server`：axum 服务，默认监听 `127.0.0.1:17870`（`SERVER_NAME` 覆盖绑定地址，`SERVER_PORT` 覆盖端口）
- `rust/state`：全局状态管理——`AppState`（with_state 注入的唯一容器：配置 + 请求统计）+ `track` 中间件自动采集
- `rust/error`：错误统一出口——`AppError`（领域错误枚举）→ `ApiError`（渲染成包裹 + 分级日志）→ `ApiResponse`（响应格式唯一定义处）

> 每个 crate 的职责、约定与用法见各自目录下的 `README.md`。

- **路由约定：一个路由一个文件夹** —— `src/routes/<name>/mod.rs` 自带 handler、参数类型与 `pub fn router()`，`src/main.rs` 只负责 `Router::new().merge(...)` 注册与启动；跨路由共用的结构放 `src/` 顶层（如 `envelope.rs` 的 `ApiResponse`）。新增路由即新建 `routes/<name>/` 文件夹 + 在 main 里 merge 一行
- 桌面壳在 setup 时用 `tauri-plugin-shell` 的 `sidecar("server")` 拉起它，退出时 kill；日志前缀 `[server]`
- sidecar 二进制由 `apps/desktop/scripts/build-sidecar.mjs` 生成：`cargo build -p server` 后拷贝为 `src-tauri/binaries/server-<target-triple>.exe`（`tauri dev` 用 debug 构建，`tauri build` 用 `--release`），`beforeDev/BuildCommand` 已自动执行
- 前端 HTTP 调用约定见 `apps/web/src/App.tsx`（`SERVER_URL`，web 端可用 `VITE_API_BASE_URL` 环境变量覆盖）；接口路由定义在 `rust/server/src/routes/` 下
- **desktop 与 web 共用同一份前端代码**：`tauri.conf.json` 的 `devUrl` 指向 web 的 dev server（5173 端口），`frontendDist` 指向 `apps/web/dist`；desktop 本身无任何前端代码

新增 Rust 包：在 `rust/` 下建目录，然后把它加进 `rust/Cargo.toml` 的 `members` 列表。

## 请求层（packages/shared/src/request）

后端统一响应包裹 `{ code, message, data }`（`code === 0` 成功，唯一定义在 `rust/error` 的 `ApiResponse`）。`@agent-packages/shared` 提供基于**原生 fetch** 的请求客户端 `createFetchClient`（零第三方依赖），返回值已剥掉包裹直接是 `data`，业务码非 0 / 网络错误 / 取消等统一抛 `ApiError`（`kind: 'business' | 'http' | 'network' | 'abort' | 'parse'`）。

### 错误码约定（前后端对齐）

| code  | HTTP | AppError 变体          | 触发方式                                         |
| ----- | ---- | ---------------------- | ------------------------------------------------ |
| 10001 | 400  | `BadRequest`           | handler 手动 / Query 提取失败（自动）            |
| 10002 | 401  | `Unauthorized`         | 手动                                             |
| 10003 | 403  | `Forbidden`            | 手动                                             |
| 10004 | 404  | `NotFound`             | 手动 / 未匹配路径 fallback（自动）               |
| 10005 | 409  | `Conflict`             | 手动                                             |
| 10006 | 405  | `MethodNotAllowed`     | method_not_allowed_fallback（自动）              |
| 10007 | 413  | `PayloadTooLarge`      | 预留，待接 RequestBodyLimitLayer                 |
| 10008 | 415  | `UnsupportedMediaType` | JSON 提取失败（自动）                            |
| 10009 | 422  | `Unprocessable`        | JSON 提取失败（自动）                            |
| 10010 | 429  | `TooManyRequests`      | 预留，待接限流层                                 |
| 10011 | 408  | `Timeout`              | 预留，待接超时层（From<Elapsed> 已就绪）         |
| 50000 | 500  | `Internal(source)`     | handler panic 兑底 / io 错误；原因只进日志不回传 |
| 50001 | 503  | `ServiceUnavailable`   | 预留                                             |

框架层错误（提取失败 / 404 / 405 / panic）已全部接管为包裹响应，前端统一归为 `kind: 'business'`；裸文本响应只剩 `/api/health` 约定。handler 返回值推荐写法：`ApiResult<Json<ApiResponse<T>>>`，错误用 `?` + `From` 自动转换。

```ts
import { ApiError, createFetchClient } from '@agent-packages/shared';

const http = createFetchClient({ baseURL: 'http://127.0.0.1:17870', timeout: 5000 });
// 返回值已剥掉 {code, message, data} 包裹，直接是 data（DTO 前端手写，与 Rust 侧结构体对齐）
const info = await http.get<ServerInfo>('/api/info');
```

能力一览：

- **取消请求**：`options.signal` 传 `AbortController.signal`；`options.timeout`（或客户端默认）超时自动取消，两者都抛 `ApiError(kind='abort')`
- **幂等接口去重**：GET/HEAD 默认开启，进行中的相同请求（method + url + query 相同）只发一次，共享同一个 Promise；`options.dedupe` 可按请求覆盖（POST 等默认不去重）
- **响应拦截器**：支持 `interceptors.request` / `interceptors.response`（拿到包裹与原始 Response，可转换/打点）
- **完整 TS 泛型**：`ApiResponse<T>`、`RequestClient`、`ApiError` 等；全部基于原生 fetch，无第三方运行时依赖

## 常用命令（根目录执行）

```bash
pnpm install        # 安装全部依赖
pnpm build          # 递归构建所有包（按拓扑序：先 shared/shared-ui 后 desktop）
pnpm typecheck      # 递归类型检查（tsc）
pnpm lint           # Oxlint（前端 TS/TSX）
pnpm lint:fix       # Oxlint 自动修复
pnpm lint:rs        # Clippy（Rust 全 workspace，警告当错误）
pnpm format         # Oxfmt 格式化（前端 + JSON/MD）
pnpm format:rs      # rustfmt 格式化（Rust 全 workspace）

# web 主应用（apps/web）
pnpm dev                                    # 浏览器开发调试（http://localhost:5173）
pnpm --filter @agent-packages/web build     # 构建产物（也是桌面壳打包用的前端产物）

# 后端接口（rust/server，单跑调接口用；与 tauri dev 同时跑会端口冲突）
pnpm server                                 # cargo run -p server，监听 http://127.0.0.1:17870
pnpm server:watch                           # 热更新：rust/ 下任何变化自动重编译重启（watchexec）

# 桌面壳（apps/desktop，先确保前端依赖已构建，tauri 命令会自动带起 web dev server）
pnpm dev:win                                # 等价于 --filter @agent-packages/desktop tauri dev
pnpm --filter @agent-packages/desktop tauri build  # 打包安装程序
```

## 前后端 DTO 约定

不引入类型生成器，两侧各自手写、保持字段对齐：

- Rust 侧结构体定义在各路由模块（如 `rust/server/src/routes/info/mod.rs` 的 `ServerInfo`），doc comment 注明前端对齐位置
- 前端 DTO 手写在消费它的文件里（如 `apps/web/src/App.tsx` 的 `interface ServerInfo`），注释指向 Rust 侧定义处
- 包裹类型 `ApiResponse<T>` 是请求管道自己的类型，定义在 `packages/shared/src/request/types.ts`，与 Rust 侧 `error::ApiResponse` 同名同构

## 组件（packages/shared-ui/src）

分层依据：**基础** = 无业务语义、零依赖的原子与布局；**高级** = 组合基础组件或引入 headless 依赖 / Portal 等复杂逻辑；**AI** = Agent 控制台场景件。

### 基础组件（原子 + 布局）

- **布局**：`Stack` / `HStack` / `VStack`（direction/gap/align/justify/wrap）、`Spacer`（撑开剩余空间）、`Divider`（发丝线，支持 `inset`）、`Container`（居中容器 sm/md/lg）、`NavBar`（毛玻璃粘顶栏）、`SectionHeader`
- **输入**：`Button`（solid/tinted/plain/gray/destructive × sm/md/lg，按压 `active:scale-[0.97]`）、`Input`（填充式，聚焦 tint 描边，`aria-invalid` 红描边）、`Switch`（iOS 规格，开启态 systemGreen）、`Checkbox`、`Slider`、`Select`
- **展示**：`Badge` / `Card` / `Avatar` / `Spinner` / `Progress` / `SegmentedControl`（等宽分段）/ `List` + `ListItem`（iOS 分组列表）

### 高级组件（组合逻辑 / headless 依赖）

- `DataTable`：TanStack Table headless，排序 / 分页，样式全部本地令牌
- `Form` + `TextField` / `SwitchField`：TanStack Form headless，校验与状态交给库，外观复用基础组件
- `Lightbox`：图片灯箱，Portal 挂载 + 滚动锁定 + 键盘导航

### AI 组件（Agent 控制台）

**单智能体**（对话/工具循环）：

- `Typewriter`：流式打字机，受控 `text` / 非受控 `ref.push()` 双模式（SSE 场景用后者）
- `useMarkdown` + Markdown 渲染：流式 fence 自动补齐，边收边渲
- `ThinkingBlock`：思维链折叠块，思考中自动展开、可手动收起
- `UsageBadge` / `UsageMeter`：token 用量徽标与上下文水位条
- `Composer`：AI 输入域（图片附件 / 模型选择 / 停止生成）+ `AttachmentList` 附件列表
- `ToolExecution`：工具调用卡片，状态驱动 running→success/error，参数与输出 0fr→1fr 披露，失败可重试
- `SkillCard`：技能卡片，图标 + 名称 + 描述 + 标签，传入 onToggle 显示启用开关（状态由调用方驱动）
- `WebSearch`：联网搜索卡片，对齐 pi 的 queries[] 模型（多角度查询、逐条 searching→done、合成答案 + 来源引用），零时序假设
- `Message` / `MessageList`：对话骨架，混合式布局（user 气泡 / assistant 通栏）+ loading 态 + 贴底滚动跟随
- `TaskPlan`：任务计划步骤清单，pending/running/done/failed 四态 + 进度头
- `ApprovalCard`：敏感操作授权卡，允许/总是允许/拒绝，状态由调用方驱动
- `ConversationList`：多会话列表，自动时间分组 + 行内重命名 + 删除
- `ErrorBanner`：失败横幅，接 ApiError.kind 分默认文案，可重试可关闭
- `Toast`：全局轻通知，ToastProvider + useToast，顶部横幅、点击即关
- `PromptSuggestions`：快捷提示词胶囊/列表，stagger 入场

**多智能体**（编排/协作，模式参考 scrimui agent-console 与 LukeW agent-management）：

- `AgentCard`：智能体状态卡，一卡一 agent：状态点 + 进行时活动 + token/耗时/成本三件套，失败卡可单独重跑（per-agent Rerun）
- `AgentTeamBoard`：团队面板（fleet roster），AgentCard 网格 + 头部汇总（运行数/待审批/总成本），选中卡切换时间线而非换页
- `AgentTree`：委派树，orchestrator → subagent 层级，缩进参考线表达并行与依赖，每节点带状态
- `AgentTimeline`：运行时间线，事件**只追加不覆盖**，重试是 linkedId 链接事件（被接替者划线保留供审计）
- `AgentHandoff`：交接回执，不是裸箭头：带走什么上下文/刻意留下什么/为什么交接，全部可审计
- `ApprovalGate`：决策门，结构化选项（2-4 个）请求拍板，内联在发起方日志流里（与 ApprovalCard 的允许/拒绝分工）
- `MemoryPanel`：记忆面板，fact/note/entity/plan 分档 + 置顶 + 检索，对齐 pi 的记忆体系

- 从包根导入：`import { Button, HStack, Switch } from '@agent-packages/shared-ui'`；新组件记得加进 `src/index.ts` barrel，并按分层归入对应小节

## 样式（Tailwind 4 + Apple HIG 主题）

- 装在 `apps/web`（`tailwindcss` + `@tailwindcss/vite`），入口 `apps/web/src/index.css`：`@import 'tailwindcss'` 后接 `@import '@agent-packages/shared-ui/styles.css'` 引入组件库主题
- **主题令牌定义在组件库里**：`packages/shared-ui/styles/`（`tokens.css` + `animations.css`，入口 `index.css`），通过 shared-ui 的 `exports["./styles.css"]` 暴露源码，由消费方的 Tailwind 编译（不预构建）
- 颜色遵循 **Apple HIG**：系统色（`--color-blue/red/...`，每色相仅明暗两个官方值）+ 语义层级（`label` 文字 / `background` 背景 / `fill` 填充 / `separator` 分隔线）+ 控件语义（`tint` 强调色 / `destructive`）。组件只用语义名（`bg-tint`、`text-secondary-label`），深浅色自动适配；换主色只改 `tokens.css` 里的 `--tint`
- 深色模式走 class 策略：消费方声明 `@custom-variant dark (&:where(.dark, .dark *))`，在 `<html>` 上挂 `.dark` 切换
- 预设动画：`animate-fade-in-up` / `animate-zoom-in` / `animate-slide-in-*` / `animate-shake` 等，强自定义缓动（`ease-out` / `ease-drawer` 已覆盖内置弱曲线），`@keyframes` 按需输出，内置 `prefers-reduced-motion` 降级。规则来自 `.agents/skills/`（emil-design-eng / apple-design）
- v4 自动检测只扫本包，不跨包：**shared-ui 组件里的 class 靠 `@source '../../../packages/shared-ui/src'` 纳入扫描**（新增组件包时同样加一条 `@source`）

## 新增一个包

1. `packages/`（或 `apps/`）下新建目录
2. `package.json` 的 `name` 用 `@agent-packages/xxx` 命名
3. `tsconfig.json` 写 `{ "extends": "../../tsconfig.base.json", "include": [...] }`
4. 声明依赖：工具链（typescript/oxlint/oxfmt）已在根目录，包内只装自己的运行时/构建依赖
5. 包之间互相引用直接用包名（如 `"@agent-packages/shared-ui": "workspace:*"`）

## 桌面壳开发说明（apps/desktop）

- **desktop 只是壳，不含前端代码**：界面即 `apps/web`，通过 `devUrl`（开发，指到 web dev server）/ `frontendDist`（打包，指到 `../../web/dist`）装载
- `tauri dev/build` 流程：`build-sidecar.mjs`（编译 axum + 拷贝 sidecar 二进制）→ 重建 `shared-ui`/`shared` → 起 web dev server 或构建（见 `src-tauri/tauri.conf.json` 的 `beforeDevCommand`/`beforeBuildCommand`）
- 日常开发在 `apps/web` 里 `pnpm dev` 用浏览器即可；只有需要验证桌面壳行为（sidecar 拉起、窗口、打包）时才跑 `tauri dev/build`
- 开发中若要热更组件库源码：另开终端跑 `pnpm --filter @agent-packages/shared-ui dev`（tsdown watch），Vite 会自动重载
- 改 Rust 代码后重跑 `tauri dev` 即可；单独调后端接口可直接 `cargo run -p server`（在 `rust/` 下），用 curl/浏览器访问 `http://127.0.0.1:17870/api/health`
