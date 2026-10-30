# 编码规范

> 工具链自动强制的部分不要靠自觉：oxfmt + oxlint + tsc strict 三道闸全绿才算完成。

## 语言与目标

- TypeScript ES2022，`moduleResolution: "bundler"`，React 19（peer >=18），`jsx: "react-jsx"`（不要 import React）。
- `tsconfig.base.json`：`strict` / `noUnusedLocals` / `noUnusedParameters` / `isolatedModules` 全开。新包继承 base，禁止放宽。

## 格式化（oxfmt，零争论）

`.oxfmtrc.json` 是唯一事实：`printWidth: 100`、`singleQuote`、`semi: true`、`trailingComma: "all"`。提交前 `pnpm format`，不手调格式。

## 命名

| 对象             | 规则                                     | 例                                 |
| ---------------- | ---------------------------------------- | ---------------------------------- |
| 组件/类型/文件夹 | PascalCase                               | `ToolExecution/`、`WebSearchQuery` |
| 函数/变量        | camelCase，事件回调用 `on` 前缀          | `onOpenSource`、`settleTool`       |
| 常量             | SCREAMING_SNAKE（模块级）                | `SEARCH_ROUNDS`                    |
| CSS class        | Tailwind 工具类 + 少量 kebab-case 自定义 | `grid-rows-[0fr]`                  |
| 布尔 props       | `is/has/should` 或形容词                 | `searchable`、`pinned`、`selected` |

## 分支写法（禁三元）

- **禁止使用三元运算符** `cond ? a : b`。分支逻辑必须落到更显式的形式，可读性优先于行数。
- 多分支取值：**查表优先**——用 `Record<枚举/字面量, 值>` 或 `Map` 把分支变成数据，新增分支不改控制流；
  键用模块级 SCREAMING_SNAKE 常量对象（见命名规范）或联合字面量枚举。
  ```ts
  // ❌ kind === 'kill' ? '刀' : kind === 'guard' ? '盾' : '好'
  // ✅
  const SEAT_MARK: Record<SeatMarkKind, string> = { kill: '刀', guard: '盾', good: '好' };
  ```
- 双分支含副作用/计算：抽函数，内部 `if` + 提前 `return`；不写三元链。
- 仅单侧需要渲染/执行：JSX 与语句均用 `cond && x`（或 `cond && undefined` 归一），空值回落用 `??`。
- 工具链层面 oxlint 只拦嵌套三元（`no-nested-ternary`）；本条以规则约束 + code review 强制，新增代码不许出现任何三元。

## React 规范

1. **受控优先**：数据由 props 进、变更由回调出；组件不持有数据副本。参考 `AgentCard`（`agent` + `onSelect`/`onRerun`）与 `WebSearch`（`queries[]` 纯渲染）。
2. **回调全可选**：不传回调 = 非交互渲染（按钮退化为文本/图标），保证纯展示场景零依赖。
3. **零时序假设**：组件不知道数据"什么时候会到"——流式文本直接绑 `text` 属性，状态推进由调用方驱动；内部计时器只属于 demo 层（`apps/web`），永远 `useRef` 收集 + 卸载清理。
4. **列表状态用稳定 id 映射**，禁止按数组下标改状态（`list.map((t) => t.id === e.id ? {...} : t)`）。
5. **导出契约**：每个组件文件同时导出 Props 类型与数据类型（如 `ToolExecutionProps` / `ToolExecutionData`），消费方不重复声明。
6. **类名一律 `cn()`**：`cn('base', cond && 'extra', className)`；组件根元素必须透传 `className`。
7. 需要可访问性语义的交互用原生元素 + `aria-*`（如折叠用 `aria-expanded`）。

## 状态与副作用

- `useEffect` 只用于订阅/清理/同步 DOM；禁止用 effect 做派生计算（直接在 render 里算）。
- 事件处理器禁止内联 setTimeout 链进组件库源码；demo 里的模拟逻辑放 App.tsx。
- 清理模式：`const timers = useRef<Timeout[]>([]); useEffect(() => () => timers.current.forEach(clearTimeout), []);`

## 类型

- 导出类型用 `export type`（`isolatedModules` 要求）；props 用 `XxxProps` 命名。
- 联合字面量优先于 enum；用 `as const` 收窄常量数组。
- 禁止 `as any`、`@ts-ignore`；确需绕过用 `@ts-expect-error` + 注释原因。

## 校验命令（提交前全绿）

```bash
pnpm format && pnpm lint
pnpm typecheck
pnpm --filter @agent-packages/shared-ui build
```
