# AgentCard 智能体状态卡

多 Agent 协作的最小单元：一卡一 agent，「谁在干什么」一眼可读。状态点 + 进行时活动文案 + 成本三件套（token / 耗时 / 费用）。纯展示组件，状态完全由调用方驱动。

## 何时使用

- 团队面板、委派树展开卡、审批流上下文等任何需要展示单个 agent 状态的地方
- 失败的子任务是一张卡，不是一次宕机：失败态提供 per-agent「重跑」（传 `onRerun` 才显示）
- 成本在卡上可见：`costUsd >= 0.5` 时金额转 orange 警示，避免失控 agent 直到账单到来才被发现

## 代码演示

### 基础用法

五种状态（running / done / waiting / idle / failed）；点击失败卡的「重跑」观察状态流转。

<DemoPreview name="AgentCardBasic" />

<<< ../demos/AgentCard/basic.tsx

## API

### AgentCardProps

| 属性        | 类型            | 默认值  | 说明                                              |
| ----------- | --------------- | ------- | ------------------------------------------------- |
| `agent`     | `AgentCardData` | —       | 卡片数据                                          |
| `selected`  | `boolean`       | `false` | 选中态（配 `onSelect`，选中卡切换时间线而非换页） |
| `onSelect`  | `() => void`    | —       | 点击回调；不传则渲染为非交互卡                    |
| `onRerun`   | `() => void`    | —       | 重跑回调；仅 `status: 'failed'` 时显示按钮        |
| `className` | `string`        | —       | 追加到根元素的类名                                |

### AgentCardData

| 属性         | 类型       | 说明                                            |
| ------------ | ---------- | ----------------------------------------------- |
| `id`         | `string`   | 唯一标识                                        |
| `name`       | `string`   | 显示名                                          |
| `role`       | `string?`  | 角色徽标；`'orchestrator'` 用蓝色徽标，其余灰色 |
| `model`      | `string?`  | 模型徽标（等宽字体）                            |
| `activity`   | `string?`  | 进行时活动文案，running 时替代状态文字          |
| `status`     | `'idle' \\ | 'running' \\                                    | 'waiting' \\ | 'done' \\ | 'failed'` | 状态：Spinner / 绿勾 / 橙色呼吸 / 红叉 / 描边空心 |
| `tokens`     | `number?`  | 累计 token                                      |
| `costUsd`    | `number?`  | 累计成本（美元）                                |
| `durationMs` | `number?`  | 运行耗时                                        |
