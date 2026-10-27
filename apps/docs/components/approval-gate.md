# ApprovalGate Agent 决策门

Agent 在关键节点请求的**结构化决策**（2-4 个选项 + 说明），对齐 pi 的 plan_mode_question 模型。pending 显示选项按钮组，decided 后收拢为结果行。

::: tip 与 ApprovalCard 的分工
`ApprovalCard` 是单次敏感操作的 允许 / 拒绝 授权（工具调用层）；`ApprovalGate` 是方案取舍类决策（任务规划层）。两者都内联在发起方的日志流里，不漂浮成全局 modal。
:::

## 何时使用

- agent 请求拍板：方案 A/B 取舍、继续与否、拆分策略等结构化选择
- fleet 头部汇总待审数（见 AgentTeamBoard 的 `pendingApprovals`），决策操作留在事件现场

## 代码演示

### 基础用法

点击任意选项提交决策，卡片收拢为结果行。

<DemoPreview name="ApprovalGateBasic" />

<<< ../demos/ApprovalGate/basic.tsx

## API

### ApprovalGateProps

| 属性              | 类型                         | 默认值     | 说明                      |
| ----------------- | ---------------------------- | ---------- | ------------------------- |
| `title`           | `string`                     | —          | 决策请求标题              |
| `description`     | `string?`                    | —          | 决策背景说明              |
| `options`         | `ApprovalOption[]`           | —          | 结构化选项（2-4 个）      |
| `status`          | `'pending' \\                | 'decided'` | `'pending'`               | 决策状态 |
| `decidedOptionId` | `string?`                    | —          | decided 时被选中的选项 id |
| `note`            | `string?`                    | —          | decided 后的补充说明      |
| `onDecide`        | `(optionId: string) => void` | —          | 决策回调                  |
| `className`       | `string`                     | —          | 追加到根元素的类名        |

### ApprovalOption

| 属性          | 类型      | 说明                     |
| ------------- | --------- | ------------------------ |
| `id`          | `string`  | 选项 id，`onDecide` 回传 |
| `label`       | `string`  | 选项标签                 |
| `description` | `string?` | 选项说明                 |
