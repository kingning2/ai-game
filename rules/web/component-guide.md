# 组件指南（使用 + 新增）

> 覆盖 `rules/common/` 的组件相关约定。组件设计理念见各组件源码 TSDoc 头注释。

## 使用组件

```tsx
import { Button, Stack, WebSearch } from '@agent-packages/shared-ui';
import '@agent-packages/shared-ui/styles.css'; // 全局一次
```

- 组件全从包根导入；样式入口全应用只引一次（含 Tailwind + 令牌 + reset）。
- 暗色：`<html class="dark">` 即可，无需逐组件处理。
- 交互组件的事件回传数据模型见组件源码导出的 Props/数据类型（TSDoc 注释即文档）。

## 组件设计契约（新增组件必须遵守）

1. **展示与状态分离**：组件是纯函数视图——数据从 props 进、意图从回调出；"数据什么时候变"由调用方决定。push 型数据（时间线、查询流、工具调用）直接绑数组属性，追加即渲染。
2. **回调可选降级**：`onXxx` 不传时渲染为静态展示（按钮隐藏或退化为图标），不得要求调用方传空函数。
3. **一对 Props/数据类型**：`XxxProps` + 领域类型（如 `AgentTimelineEvent`）同文件导出；字段可缺省的用 `?` 并在 TSDoc 标注默认行为。
4. **语义状态枚举**：跨组件复用同一套状态词：`'idle' | 'running' | 'waiting' | 'done' | 'failed'`；事件类组件用 `kind` 枚举（spawn/tool/handoff/approval/retry/error/done/note）。
5. **历史只追加不覆盖**：重试/接替用链接字段（`linkedId`）关联旧条目，旧条目划线保留——审计需要历史。
6. **折叠/展开**：`grid-template-rows` 过渡 + `aria-expanded`，参考 ToolExecution/WebSearch 实现。
7. **敏感操作**：单次操作授权用 `ApprovalCard`（允许/拒绝）；方案取舍用 `ApprovalGate`（结构化选项）。都内联在事件流里，不弹全局 modal。
8. 单智能体组件（对话循环）与多智能体组件（编排协作）在 README 与 `index.ts` 分组注释中维护，不混排。

## 新增组件全流程（ECC 式 checklist，缺一不可）

```bash
# 1. 组件本体
packages/shared-ui/src/<Name>/index.tsx      # 组件 + Props/数据类型 + TSDoc
packages/shared-ui/src/index.ts              # barrel 导出（按分组注释放对区块）

# 2. 演示场（交互类组件必须）
apps/web/src/App.tsx                         # 新 Section：状态推进 + 重置按钮

# 3. 登记
README.md                                    # 组件清单（重要组件）

# 4. 验收
pnpm build
typecheck
pnpm dev   # 明暗两套肉眼过一遍
```

## Demo 编写规则（apps/web）

- demo 是**使用范例**，不是测试：展示典型 props 组合与状态流转，逻辑 ≤ 60 行。
- demo 内允许 `useState`/定时器模拟数据推进（组件库源码内禁止）。
- 演示逻辑放 `App.tsx` 的 Section 或 `src/<domain>/`，不进组件库。
