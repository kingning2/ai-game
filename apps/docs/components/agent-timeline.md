# AgentTimeline 运行时间线

Agent 运行事件的追加式时间线：左侧时间轴 + 竖轨串联，事件**只追加、不覆盖**。重试是追加一条 `linkedId` 指向被接替事件的链接事件，被接替者划线保留 —— 调试需要的历史就在这里。

## 何时使用

- 作为多 Agent 组件的容器/底座：spawn、tool、handoff、approval、retry、error 等事件按需 push
- 配合 AgentTeamBoard：选中卡片切换对应 agent 的时间线
- `onOpenEvent` 可选，用于点击事件跳转（如定位到对应 agent 卡片）

## 代码演示

### 基础用法

包含全部事件类型：注意 e4（webhook 超时）被 e5（重跑）接替后划线保留，不再显示为当前失败。

<DemoPreview name="AgentTimelineBasic" />

<<< ../demos/AgentTimeline/basic.tsx

## API

### AgentTimelineProps

| 属性          | 类型                                  | 默认值 | 说明                         |
| ------------- | ------------------------------------- | ------ | ---------------------------- |
| `events`      | `AgentTimelineEvent[]`                | —      | 事件数组，由调用方 push 驱动 |
| `title`       | `string`                              | —      | 时间线标题                   |
| `onOpenEvent` | `(event: AgentTimelineEvent) => void` | —      | 事件点击回调                 |
| `className`   | `string`                              | —      | 追加到根元素的类名           |

### AgentTimelineEvent

| 属性       | 类型        | 说明                                                 |
| ---------- | ----------- | ---------------------------------------------------- |
| `id`       | `string`    | 唯一 id                                              |
| `kind`     | `'spawn' \\ | 'tool' \\                                            | 'handoff' \\ | 'approval' \\ | 'retry' \\ | 'error' \\ | 'done' \\ | 'note'` | 事件类型（决定图标与语义色） |
| `title`    | `string`    | 事件标题                                             |
| `detail`   | `string?`   | 补充说明（工具名 / 耗时 / 错误信息）                 |
| `time`     | `string?`   | 发生时间，如 `'09:40:41'`                            |
| `linkedId` | `string?`   | 重试/接替指向的原事件 id（追加链接事件，不覆盖历史） |
