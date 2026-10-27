# AgentHandoff Agent 交接回执

Agent 间交接的可审计回执：不是裸箭头 —— 展示**带走了什么上下文、刻意留下了什么、为什么交接**。内联组件，渲染在时间线/消息流里，发生在哪就展示在哪。

## 何时使用

- orchestrator 把任务移交给专家 agent 时，回显上下文交接清单
- 审计需求：读者需要知道下一个 agent 实际看到了什么（`carried`）、明确排除了什么（`withheld`）

## 代码演示

### 基础用法

Researcher → Writer 的交接：携带 3 项上下文（绿勾），留下 2 项（灰横线）。

<DemoPreview name="AgentHandoffBasic" />

<<< ../demos/AgentHandoff/basic.tsx

## API

### AgentHandoffProps

| 属性        | 类型       | 默认值 | 说明                                 |
| ----------- | ---------- | ------ | ------------------------------------ |
| `from`      | `string`   | —      | 交接发起方名称                       |
| `to`        | `string`   | —      | 接收方名称                           |
| `reason`    | `string?`  | —      | 交接原因                             |
| `carried`   | `string[]` | `[]`   | 携带给下一棒的上下文清单（绿勾）     |
| `withheld`  | `string[]` | `[]`   | 刻意留下的内容清单（灰横线，审计用） |
| `time`      | `string?`  | —      | 发生时间，如 `'09:40:41'`            |
| `className` | `string`   | —      | 追加到根元素的类名                   |
