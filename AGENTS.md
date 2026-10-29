# AGENTS.md — 项目规范总纲

> 本文件是所有 AI agent 与贡献者的**必读入口**。具体规则在 `rules/` 分层目录中，按需加载。
> 分层模式参考 [ECC](https://github.com/affaan-m/ECC)：通用层 + 领域层，**具体层覆盖通用层**（specific overrides general）。

## 项目是什么

`agent-packages`：Apple HIG 风格、令牌驱动的 React 组件库 monorepo，面向 AI Agent 控制台场景。

| 包                          | 路径                 | 职责                              |
| --------------------------- | -------------------- | --------------------------------- |
| `@agent-packages/shared`    | `packages/shared`    | 纯逻辑（无 React）：类型、请求    |
| `@agent-packages/shared-ui` | `packages/shared-ui` | 组件库本体（React + Tailwind v4） |
| web 演示场                  | `apps/web`           | 全组件演示（App.tsx 分 Section）  |
| 桌面端                      | `apps/desktop`       | Tauri 壳                          |

## 核心约束（永远生效）

1. **pnpm**，包管理器锁定 `pnpm@12.14.0`；禁止 npm/yarn。
2. **TS strict**：`tsconfig.base.json` 开启 `strict` + `noUnusedLocals` + `noUnusedParameters`，不许 `as any` 逃逸。
3. **零硬编码色值/数值**：颜色、阴影、缓动一律走 `packages/shared-ui/styles/tokens.css` 令牌；间距走 4px 阶梯；明暗两套主题下都必须可读。
4. **组件零业务逻辑、零内部数据状态**：数据驱动组件受控 + 可选回调（参考 AgentCard/ToolExecution 的契约）。
5. **flex 优先**：用 `Stack`，能不用 grid 就不用（折叠动画等特殊场景除外）。
6. **动画只动 composite 属性**（transform/opacity），一律 `ease-spring`，禁止从 `scale(0)` 入场（用 `≥0.9 + opacity`）。
7. **`cn()` 拼类名**，禁止模板字符串手拼条件类。

## Ponytail（编码纪律，永远生效）

所有编码工作（写/改/重构/审查/选依赖）默认遵循 `ponytail` skill（user-level，`~/.agents/skills/ponytail`），默认 full 强度，会话内持续：

1. **先爬梯子再写码**：①这功能该存在吗（YAGNI）→ ②仓库里已有？→ ③标准库？→ ④平台原生？→ ⑤已有依赖？→ ⑥一行能写吗？→ ⑦才写最小可用代码。
2. **Bug 修复修根因**：改前 grep 所有调用方，在共享路径修一次，不逐调用方打补丁；只修 ticket 提到的那条路会让兄弟调用方继续坏。
3. **禁止**：单实现接口、单产品工厂、为永不变化的值做配置、为「以后」留的脚手架；删除优于新增，少文件、最短可用 diff。
4. **刻意简化处留 `ponytail:` 注释**，注明天花板与升级路径（如 `// ponytail: O(n²) 扫描，列表超千条换索引`）。
5. **永不偷懒的部分**：信任边界的校验、防数据丢失的错误处理、安全、可达性基础（WCAG/触控目标/焦点）、用户明确要求的范围；**理解问题也永不偷懒**——先读完所有触点、追完真实数据流，再选梯级。
6. **非平凡逻辑**（分支/循环/解析器/钱与安全路径）留一个最小可运行检查（一行 assert 或一个小 test），不用框架不建套件；琐碎改动不用。
7. **输出**：代码优先，未被要求的解释 ≤3 行；用户明确要求的报告/讲解给全。

强度切换 `/ponytail lite|full|ultra`。本节约束的是「做多少」，`rules/` 约束「怎么做」，冲突时验收底线（构建/类型/可达性）不妥协。

## 规则索引

| 文件                                                                     | 内容                                                                      | 什么时候读      |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------------- | --------------- |
| [`rules/common/project-structure.md`](rules/common/project-structure.md) | 文件放哪个文件夹、命名、barrel 导出、包间纪律（依赖红线/导出/样式所有权） | 新建任何文件前  |
| [`rules/common/coding-style.md`](rules/common/coding-style.md)           | TS/React 编码规范、格式化、校验命令                                       | 写代码前        |
| [`rules/web/ui-design.md`](rules/web/ui-design.md)                       | 令牌三层、Apple HIG 用色、动效、布局、反馈面/overlay 判断力               | 做任何 UI 前    |
| [`rules/web/component-guide.md`](rules/web/component-guide.md)           | 组件怎么用、怎么新增一个组件（全流程 checklist）                          | 使用/新增组件时 |

## Rust 代码规则（强制）

本项目装有 Rust skills（`.agents/skills/`），碰 Rust 代码时必须遵守：

1. **任何 Rust 问题先读 `rust-router` skill**，按它的三层路由（语言机制 → 设计 → 领域约束）走，禁止直接给表面修复（如遇到 E0382 就建议 `.clone()`）。
2. **workspace 结构约束读 `rust-cargo-workspace-designer` skill**：改根 Cargo.toml / 新建 crate / 加依赖前必读；交付前跑它自带的机械检查脚本（可执行文件放在 /tmp/，不存在则按 skill 说明用 rustc 重新编译）：`/tmp/manifest_lint . --strict && /tmp/dep_audit . --strict`，有 finding 必须修复。
3. **clippy lint 表**：workspace 根 `Cargo.toml` 的 `[workspace.lints]` 及配套 `clippy.toml` 按 `rust-cargo-workspace-designer/references/codex-clippy-lints.md` 配置（来自 openai/codex 的 37 条 deny 清单）；member 必须继承 `[lints] workspace = true`，禁止局部 `#[allow]` 逃逸。
4. 上述 skill 与本文件冲突时，**碰 Rust 代码以 skill 为准**，其余（构建/验收流程）以本文件为准。

## UI/UX 规则（强制）

本项目装有 UI/UX skills（`.agents/skills/`），做任何 UI 相关工作必须遵守：

1. **任何 UI 设计/组件/视觉决策先查 `ui-ux-pro-max` skill**：用它的检索工具按领域查规则后再动手——`python .agents/skills/ui-ux-pro-max/scripts/search.py "<query>" --domain <ux|typography|color|chart>`，已知实现栈加 `--stack react`；10 级优先级清单里可达性与触控交互是 CRITICAL，禁止跳过检索凭感觉写样式。
2. **判断力规则读 `dsh-client-ui-ux`**（已提炼进 `rules/web/ui-design.md`）：反馈面三选一（Toast/就地 notice/空态）、overlay 三验证、loading 纪律、对齐轴、字重 ≤500。
3. 设计系统/令牌架构读 `design-system-patterns`；Tailwind v4 细节读 `tailwind-design-system`。
4. skill 给出的通用色值/字号/阴影建议，必须转成 `tokens.css` 令牌后才能进组件（零硬编码约束优先）；其余与本文件、`rules/web/ui-design.md` 冲突时，以本仓库规则为准。

## Agent skills（`.agents/skills/`）

- 已装 skill（多包治理、Rust、UI/UX 等）全部登记在根 `skills-lock.json`（source + sha256），新装/卸载必须同步更新 lock
- skill 提供领域方法与判断力；**与本文件、`rules/` 冲突时以本仓库规则为准**（Rust 领域例外，见上节）
- skill 在会话启动时加载；新增 skill 对当次会话不生效

## 常用命令

```bash
pnpm build          # 全 workspace 构建
pnpm typecheck      # 全 workspace tsc --noEmit
pnpm lint           # oxlint（type-aware，依赖 oxlint-tsgolint）
pnpm lint:fix       # oxlint --fix
pnpm format         # oxfmt
pnpm dev            # web 演示场（Vite）
```

## 工具链备注

- 任务编排用 Turborepo：`pnpm build` / `pnpm typecheck` 走 `turbo run`（`turbo.json` 定义 `build`→`^build` 依赖与 `dist/**` 缓存，`typecheck` 依赖上游产物）；包自身脚本不要绕过这两个任务名
- 依赖版本唯一事实在 `pnpm-workspace.yaml` 的 `catalog:`，新增依赖一律用 `catalog:` 引用，禁止在 package.json 手写版本号
- 多包治理参考已装 skill：`monorepo-management`（工作区划分/依赖图）、`turborepo-caching`（构建缓存）；UI/UX 判断力参考 `dsh-client-ui-ux`（反馈面/overlay/对齐轴）、`ui-ux-pro-max`（可达性/设计检索，强制见 UI 节）、`design-system-patterns`；skill 通用建议与本文件或 `rules/` 冲突时，一律以本仓库规则为准（如 changesets 发布、tsup、npm workspaces 不适用于本项目：私有包、tsdown、pnpm catalog）
- 全链路 oxc：oxlint（type-aware）+ oxfmt + Vite 8/Rolldown（oxc transform），无 Babel、无 ESLint/Prettier 残留
- 项目扩展 `.pi/extensions/repo-guard.ts`：硬编码色值拦截、rules 自动注入、验收提醒（下个会话生效；逃生舱 `token-guard:allow`）
- React Compiler 已在 `apps/web` 启用（`@vitejs/plugin-react` v6.1+ `compiler: true`，底层为 `oxc-transform-react` 的 Rust 实现）
- oxlint 内置 22 条 React Compiler 校验规则（`react/purity`、`immutability`、`set-state-in-render` 等，随 correctness/suspicious 分类生效），违反 Rules of React 的代码会被 compiler 静默跳过 memoization，lint 是唯一防线

## 交付验收（每次改动后）

1. `pnpm build && pnpm typecheck` 全绿（Turborepo 拓扑构建 + 缓存）
2. 动了 UI → `pnpm dev` 肉眼过一遍明暗两套主题
