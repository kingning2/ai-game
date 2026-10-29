# 文件放置规范

> 原则：**一类文件只有一个家**。放错位置的文件等于不存在的文件。

## Monorepo 布局

```
agent-packages/
├── AGENTS.md                  # 规范入口（agent 必读）
├── rules/                     # 工程规范（分层见 rules/README.md）
├── packages/
│   ├── shared/                # 纯逻辑：类型、请求，禁止 import react
│   └── shared-ui/             # 组件库本体
│       ├── src/
│       │   ├── <Name>/index.tsx   # 一个组件一个文件夹（PascalCase）
│       │   ├── layout/            # 布局组件特例：Stack/Divider/Container/NavBar/SectionHeader
│       │   ├── cn.ts              # 类名工具
│       │   └── index.ts           # 唯一 barrel：所有公开导出
│       └── styles/
│           ├── index.css          # 样式唯一入口（tailwind + tokens + reset）
│           ├── tokens.css         # 设计令牌（明暗两套），唯一允许出现色值的地方
│           ├── animations.css     # 预设动画
│           └── components.css     # 组件级全局样式（少用）
├── apps/
│   ├── web/                   # 演示场：src/App.tsx 按 Section 编排全部组件
│   └── desktop/               # Tauri
└── rust/                      # Rust 服务端
```

## 放置判定表

| 你要新建的东西 | 放这里                                                       | 命名                        |
| -------------- | ------------------------------------------------------------ | --------------------------- |
| UI 组件        | `packages/shared-ui/src/<Name>/index.tsx`                    | 文件夹 PascalCase，导出同名 |
| 组件用到的类型 | 同组件文件内导出（小类型）或 `src/<Name>/types.ts`（大类型） | PascalCase                  |
| 纯工具函数     | `packages/shared/src/`                                       | camelCase                   |
| 设计令牌/动画  | `packages/shared-ui/styles/`                                 | kebab-case class            |
| 演示场示例     | `apps/web/src/App.tsx` 新 `<Section>`                        | index 递增                  |
| 布局类组件     | `packages/shared-ui/src/layout/`                             | PascalCase                  |

## 新组件交付 checklist（缺一不可）

1. `packages/shared-ui/src/<Name>/index.tsx` — 组件 + 类型
2. `packages/shared-ui/src/index.ts` — barrel 追加导出（按 布局/基础/展示/…/AI 分组注释放对区块）
3. `apps/web/src/App.tsx` — 新增 Section（可选，交互类组件必须）
4. `README.md` 组件清单登记（可选，重要组件）

## 禁止事项

- 禁止在 `apps/*` 里写新组件（apps 只消费 `@agent-packages/shared-ui`）
- 禁止绕过 barrel 直接深路径导入（`@agent-packages/shared-ui` 根导入是唯一方式；样式经 `./styles.css`）
- 禁止在 `packages/shared` 里 import react / DOM
- 禁止在组件文件里写 `*.css`（样式统一走 styles/ + Tailwind 工具类）

## 包间纪律（借鉴 deepseek-harness packages 规范）

> 来源：deepseek-harness `packages/client/AGENTS.md` + `docs/web-styling.md`（提炼自 R3 调研）。

### 单向依赖红线

```
shared ← shared-ui ← apps/web ← apps/desktop
```

- 上层可 import 下层，下层禁止 import 上层；**平级包之间禁止 runtime 互相 import / re-export**（apps/web 与 apps/desktop 互不引用）。
- 跨组件复用的代码只进窄的静态 owner（shared / shared-ui），禁止「从兄弟包借一下」。

### 导出纪律

- 公开 API 最小化：实现组件、helpers、常量保持模块内部；barrel 只导出契约面。
- **不为「方便测试」拓宽公开 API**——同包测试直接相对路径导入内部文件。
- 类型从定义处导出一次，禁止转发 re-export 链。

### 组件零 ctx / 零订阅机器

- 业务组件纯 props：数据三通道 = 调用方 props / 组件内部 state / 可选回调上报；禁止订阅全局 store、禁止消费/提供 context。本仓库同义规则：组件零业务逻辑、零内部数据状态（AGENTS.md 核心约束 4）。
- web 层（apps）纯展示：业务逻辑不漏进展示组件，留在调用方或 shared。

### 样式所有权（细web-styling）

- `tokens.css` 独占色值/阴影/缓动；组件只消费语义工具类（`bg-fill` / `text-secondary-label`），禁止复制静态色值。
- 主题分支（`dark:` 前缀）不进组件——令牌层翻转，组件无感知（详见 ui-design.md）。
- 组件本地 custom property 仅限组件契约（如 `--slider-fill`），挂在组件根元素，不越过组件边界影响子树。

### 依赖声明精确

- 运行时依赖进 `dependencies`；仅构建/类型用进 `devDependencies`（类型包一律 dev）。
- `peerDependencies` 只声明宿主必须自备的运行时前提（react >= 18），配 `devDependencies` 供开发。
- 新依赖一律 `catalog:` 引用（版本唯一事实在 `pnpm-workspace.yaml`）。
