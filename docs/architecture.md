# agent-packages 前端架构总览

> pnpm + Turborepo monorepo，分层架构：**UI 资产层 → 纯逻辑层 → 应用层**，Rust server 做后端，Tauri 可打包桌面端。

## 一、包依赖关系（自底向上）

```
packages/shared       （纯逻辑，无 React：fetch/SSE/WS、类型守卫、错误分类）
        ↑
packages/game         （对局基类：WerewolfGame、回放状态机，无 React、无网络）
        ↑                      ↑
packages/shared-ui    （React 组件库 + Tailwind v4 + 设计令牌）
        ↑                      ↑
        apps/web（业务应用）← apps/desktop（Tauri 壳，仅打包）
                                ↓
                        rust/server（/api 后端）
```

分层纪律：每层只依赖下一层；UI 不碰网络、逻辑不碰 React、组件零业务逻辑。

## 二、文件夹结构

```
agent-packages/
├── AGENTS.md                      # agent 规范总纲（必读入口）
├── README.md
├── package.json                   # 根脚本（turbo run build/typecheck）
├── pnpm-workspace.yaml            # workspace + catalog 依赖版本唯一事实
├── turbo.json                     # build→^build 拓扑、typecheck、dist 缓存
├── tsconfig.base.json             # strict + noUnusedLocals/Parameters
├── skills-lock.json               # 已装 agent skills 登记（source + sha256）
│
├── rules/                         # 分层规范文档（具体层覆盖通用层）
│   ├── common/
│   │   ├── coding-style.md        # TS/React 编码规范、校验命令
│   │   └── project-structure.md   # 文件放置、命名、barrel、包间纪律
│   └── web/
│       ├── ui-design.md           # 令牌三层、HIG 用色、动效、布局
│       ├── component-guide.md     # 组件使用/新增全流程 checklist
│       └── app-design.md
│
├── packages/
│   ├── shared/                    # @agent-packages/shared —— 基础设施层（无 React）
│   │   ├── index.ts               # barrel 导出
│   │   ├── src/
│   │   │   ├── is.ts              # 类型守卫（isArray/isObject…）
│   │   │   └── request/
│   │   │       ├── fetch.ts       # createFetchClient（ApiResponse 信封自动解包）
│   │   │       ├── sse.ts         # createSseStream
│   │   │       ├── ws.ts          # WebSocket 通道
│   │   │       ├── types.ts       # 请求类型 / ApiError 故障分类
│   │   │       ├── utils.ts
│   │   │       └── index.ts
│   │   ├── tsdown.config.ts       # tsdown 构建
│   │   └── package.json
│   │
│   ├── game/                      # @agent-packages/game —— 领域逻辑层（无 React、无网络）
│   │   ├── index.ts
│   │   ├── src/
│   │   │   ├── enums.ts           # 动作/事件/角色/阵营枚举
│   │   │   ├── types.ts           # GameEventDto / GameFeed / ProjectedStage
│   │   │   ├── playback.ts        # 回放节奏纯函数（停留时长、舞台投影、口播）
│   │   │   ├── game.ts            # WerewolfGame 基类（事件队列、seq 去重、
│   │   │   │                      #   playhead 回放、倒计时、快照订阅）
│   │   │   └── index.ts
│   │   ├── tsdown.config.ts
│   │   └── package.json
│   │
│   └── shared-ui/                 # @agent-packages/shared-ui —— 设计系统层（React + Tailwind v4）
│       ├── COMPONENTS.md / README.md
│       ├── src/
│       │   ├── index.ts           # barrel：布局 → 基础 → 展示 → AI 场景
│       │   ├── cn.ts              # cn() 类名拼接（禁模板字符串手拼）
│       │   ├── layout/            # Stack（flex 优先）/ Divider / NavBar
│       │   ├── Button/ Input/ Slider/          # 基础组件
│       │   ├── Badge/ Card/ Avatar/ SegmentedControl/
│       │   ├── SettingsPanel/ Spinner/ List/   # 展示组件
│       │   └── WerewolfStage/     # AI 场景：index / PlayerSeat / RoleBadge
│       ├── styles/
│       │   ├── tokens.css         # 三层令牌之根：iOS 12 色相 + label/background
│       │   │                      #   语义色，:root / .dark 双主题，@theme inline
│       │   ├── components.css
│       │   ├── animations.css     # 只动 transform/opacity，统一 ease-spring
│       │   └── index.css
│       ├── vitest.setup.ts
│       ├── tsdown.config.ts
│       └── package.json
│
├── apps/
│   ├── web/                       # @agent-packages/web —— 业务应用（AI 狼人杀对战）
│   │   ├── index.html
│   │   ├── vite.config.ts         # Vite 8/Rolldown + React Compiler + /api 反代
│   │   ├── public/
│   │   │   ├── avatars/           # 各模型头像（deepseek/openai/claude/qwen…）
│   │   │   └── werewolf/          # 夜晚背景图等静态资源
│   │   └── src/
│   │       ├── main.tsx / App.tsx # react-router v8 集中式路由
│   │       │                      #   /werewolf 大厅 → /setup/:boardId → /play → settings
│   │       ├── index.css
│   │       ├── layout/
│   │       │   └── AppLayout.tsx  # 应用壳（NavBar + 返回大厅）
│   │       ├── apis/              # 数据通道（全站收口）
│   │       │   ├── http.ts        # 唯一 fetch 实例 http + SERVER_ERROR_CODE
│   │       │   │                  #   （镜像 rust AppError 错误码，两端同步维护）
│   │       │   ├── werewolf.ts    # 一函数一端点 + SSE 流
│   │       │   ├── settings.ts    # 设置/名册端点
│   │       │   └── types.ts       # DTO 契约
│   │       ├── werewolf/          # 业务域
│   │       │   ├── boards.ts      # 板子定义 BOARDS
│   │       │   ├── types.ts / speech.ts / CaptionBar.tsx
│   │       │   ├── lobby/         # LobbyPage / GameHubPage / RoomSetup（创建向导）
│   │       │   └── arena/         # 对局页
│   │       │       ├── WarfieldPage.tsx     # 对局/观战主页面
│   │       │       ├── useGameStream.ts     # SSE → WerewolfGame.ingest →
│   │       │       │                        #   useSyncExternalStore 快照（断线 3s 重连）
│   │       │       ├── HostPanel / SpeechPanel / MvpPanel
│   │       │       ├── useRecorder.tsx      # 录制
│   │       │       └── mock.ts
│   │       └── settings/          # SettingsPage / SettingsModal / RosterSettingsPage /
│   │                              #   RosterManager / agentConfig
│   │
│   └── desktop/                   # @agent-packages/desktop —— Tauri v2 壳（打包 web）
│       ├── package.json           # 仅 tauri CLI
│       ├── scripts/
│       │   └── build-sidecar.mjs  # server 打包为 sidecar 二进制
│       └── src-tauri/
│           ├── tauri.conf.json / Cargo.toml / build.rs
│           ├── capabilities/default.json
│           ├── binaries/          # server-x86_64-pc-windows-msvc.exe
│           └── src/ main.rs / lib.rs
│
└── rust/                          # 后端（Rust workspace）
    ├── Cargo.toml / rust-toolchain.toml
    ├── server/                    # axum 服务：routes/（werewolf、rooms、roster、
    │                              #   leaderboard、tts、agent_config、health…）
    │                              #   error/app_error.rs（错误码源头）
    ├── agent/ db/ games/ memory/ state/ tts/ utils/
    └── …
```

## 三、各层职责

### 1. 基础设施层 `packages/shared`
- `createFetchClient`：ApiResponse 信封自动解包、超时、故障分类（ApiError → ServerFailure）
- `createSseStream` / WS 通道、`is.ts` 类型守卫。零 React 依赖。

### 2. 领域逻辑层 `packages/game`
- `WerewolfGame` 对局基类：事件队列、按 seq 去重累积、playhead 回放、倒计时、快照订阅
- 框架无关、传输无关——调用方拉数据后 `ingest()` 喂入，状态推进由调用方驱动

### 3. 设计系统层 `packages/shared-ui`
- Apple HIG 风格组件 16 组，barrel 导出，组件受控 + 回调、零内部数据状态
- 令牌三层：`tokens.css`（原始变量 + 语义层 `@theme inline`）→ `components.css` / `animations.css` → 组件只用语义类名
- 约束：零硬编码色值/数值；颜色走透明度修饰符（Apple 做法）；动画只动 transform/opacity、统一 `ease-spring`、禁止 scale(0) 入场；`cn()` 拼类名

### 4. 应用层 `apps/web`
- **路由**：react-router v8，`App.tsx` 集中式路由；页面间经 `location.state` 传递（发牌结果 + 房间号），刷新降级到默认板子
- **单向数据流**：

```
apis/http.ts（唯一 fetch 实例 + 错误码镜像）
    → apis/werewolf.ts（一函数一端点 + SSE）
    → useGameStream（SSE → WerewolfGame.ingest → useSyncExternalStore 快照）
    → 页面只消费快照与 phase/failure，不接触传输细节
```

- 后端契约：错误码 `SERVER_ERROR_CODE` 与 `rust/server/src/error/app_error.rs` 手工两端同步

## 四、工程化选型

| 维度 | 选型 |
|---|---|
| 包管理 | pnpm 12.14（`catalog:` 统一版本，禁止手写版本号） |
| 任务编排 | Turborepo（`build`→`^build` 拓扑依赖 + `dist/**` 缓存） |
| 库构建 | tsdown |
| 应用构建 | Vite 8 / Rolldown（oxc transform，无 Babel） |
| React | 19 + React Compiler（oxc 实现，apps/web 开启） |
| CSS | Tailwind v4（`@theme inline` 令牌直连） |
| 质量 | TS strict（无 `as any`）+ oxlint（type-aware）+ oxfmt，全链路 oxc |
| 测试 | Vitest（shared-ui 配 vitest.setup.ts） |
| 桌面端 | Tauri v2，Rust server 以 sidecar 二进制打包 |

## 五、一句话概括

**令牌驱动的 Apple HIG 组件库（shared-ui）+ 框架无关游戏引擎（game）+ 严格收口的数据通道（shared），由 apps/web 组合成 AI 狼人杀对战应用；Rust server 做后端，Tauri 打包桌面端。** 核心特色是分层纪律：每层只依赖下一层，UI 不碰网络、逻辑不碰 React、组件零业务逻辑。
