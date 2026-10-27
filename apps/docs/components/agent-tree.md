# AgentTree 智能体委派树

orchestrator → subagent 的层级委派关系：嵌套缩进 + 参考线表达并行与依赖，每节点带状态点与进行时活动。状态语义与 AgentCard / TaskPlan 完全一致。

## 何时使用

- 展示编排器把任务拆给了哪些子 agent、各自进展如何
- 多根传入可表达并行编排（多个 orchestrator 各管一支）
- 树结构完全由调用方维护，组件零内部状态、零时序假设

## 代码演示

### 基础用法

一个 orchestrator 三个子任务：已完成 / 运行中（带活动文案）/ 失败（带原因）。

<DemoPreview name="AgentTreeBasic" />

<<< ../demos/AgentTree/basic.tsx

## API

### AgentTreeProps

| 属性        | 类型              | 默认值 | 说明                                          |
| ----------- | ----------------- | ------ | --------------------------------------------- |
| `nodes`     | `AgentTreeNode[]` | —      | 树根数组（通常一个 orchestrator，可多根并行） |
| `className` | `string`          | —      | 追加到根元素的类名                            |

### AgentTreeNode

| 属性       | 类型               | 说明                                        |
| ---------- | ------------------ | ------------------------------------------- |
| `id`       | `string`           | 唯一标识                                    |
| `name`     | `string`           | 显示名                                      |
| `role`     | `string?`          | 角色徽标；`'orchestrator'` 用蓝色，其余灰色 |
| `status`   | `'idle' \\         | 'running' \\                                | 'waiting' \\ | 'done' \\ | 'failed'` | 节点状态 |
| `activity` | `string?`          | 进行时活动文案（running 时显示在行尾）      |
| `children` | `AgentTreeNode[]?` | 子委派（下一层 agent）                      |
