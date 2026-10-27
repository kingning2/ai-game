# AGENTS.md — 项目规范总纲

> 本文件是所有 AI agent 与贡献者的**必读入口**。具体规则在 `rules/` 分层目录中，按需加载。
> 分层模式参考 [ECC](https://github.com/affaan-m/ECC)：通用层 + 领域层，**具体层覆盖通用层**（specific overrides general）。

## 项目是什么

`agent-packages`：Apple HIG 风格、令牌驱动的 React 组件库 monorepo，面向 AI Agent 控制台场景。

| 包                          | 路径                 | 职责                                |
| --------------------------- | -------------------- | ----------------------------------- |
| `@agent-packages/shared`    | `packages/shared`    | 纯逻辑（无 React）：类型、请求      |
| `@agent-packages/shared-ui` | `packages/shared-ui` | 组件库本体（React + Tailwind v4）   |
| web 演示场                  | `apps/web`           | 全组件演示（App.tsx 分 Section）    |
| 文档站                      | `apps/docs`          | VitePress，每组件一页 + 可交互 demo |
| 桌面端                      | `apps/desktop`       | Tauri 壳                            |

## 核心约束（永远生效）

1. **pnpm**，包管理器锁定 `pnpm@12.14.0`；禁止 npm/yarn。
2. **TS strict**：`tsconfig.base.json` 开启 `strict` + `noUnusedLocals` + `noUnusedParameters`，不许 `as any` 逃逸。
3. **零硬编码色值/数值**：颜色、阴影、缓动一律走 `packages/shared-ui/styles/tokens.css` 令牌；间距走 4px 阶梯。
4. **组件零业务逻辑、零内部数据状态**：数据驱动组件受控 + 可选回调（参考 AgentCard/ToolExecution 的契约）。
5. **flex 优先**：用 `Stack`，能不用 grid 就不用（折叠动画等特殊场景除外）。
6. **动画只动 composite 属性**（transform/opacity），一律 `ease-spring`，禁止从 `scale(0)` 入场（用 `≥0.9 + opacity`）。
7. **`cn()` 拼类名**，禁止模板字符串手拼条件类。

## 规则索引

| 文件                                                                     | 内容                                             | 什么时候读      |
| ------------------------------------------------------------------------ | ------------------------------------------------ | --------------- |
| [`rules/common/project-structure.md`](rules/common/project-structure.md) | 文件放哪个文件夹、命名、barrel 导出              | 新建任何文件前  |
| [`rules/common/coding-style.md`](rules/common/coding-style.md)           | TS/React 编码规范、格式化、校验命令              | 写代码前        |
| [`rules/web/ui-design.md`](rules/web/ui-design.md)                       | 令牌三层、Apple HIG 用色、动效、布局             | 做任何 UI 前    |
| [`rules/web/component-guide.md`](rules/web/component-guide.md)           | 组件怎么用、怎么新增一个组件（全流程 checklist） | 使用/新增组件时 |

## 常用命令

```bash
pnpm build          # 全 workspace 构建
pnpm typecheck      # 全 workspace tsc --noEmit
pnpm lint           # oxlint（type-aware，依赖 oxlint-tsgolint）
pnpm lint:fix       # oxlint --fix
pnpm format         # oxfmt
pnpm dev            # web 演示场（Vite）
pnpm dev:docs       # 文档站（VitePress dev）
```

## 工具链备注

- 全链路 oxc：oxlint（type-aware）+ oxfmt + Vite 8/Rolldown（oxc transform），无 Babel、无 ESLint/Prettier 残留
- React Compiler 已在 `apps/web` 与 `apps/docs` 启用（`@vitejs/plugin-react` v6.1+ `compiler: true`，底层为 `oxc-transform-react` 的 Rust 实现；VitePress 需 exclude Vue SFC 虚拟模块）
- oxlint 内置 22 条 React Compiler 校验规则（`react/purity`、`immutability`、`set-state-in-render` 等，随 correctness/suspicious 分类生效），违反 Rules of React 的代码会被 compiler 静默跳过 memoization，lint 是唯一防线

## 交付验收（每次改动后）

1. `pnpm --filter @agent-packages/shared-ui build && pnpm typecheck` 全绿
2. 动了组件 → 文档站 `pnpm --filter @agent-packages/docs exec vitepress build` 全绿
3. 动了 UI → `pnpm dev` 肉眼过一遍明暗两套主题
