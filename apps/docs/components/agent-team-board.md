# AgentTeamBoard 智能体团队面板

fleet 级「谁在干什么」一屏可读：AgentCard 网格 + 头部汇总（运行数 / 等待数 / 失败数 / 总 token / 总成本）。待审批数在头部汇总计数，但审批操作内联在对应 agent 的时间线里。

## 何时使用

- 多 agent 并行任务的监控首页：选中卡片切换右侧时间线，永不换页
- 成本双显示：单卡显示 per-run 成本，头部汇总 fleet 总成本（`≥ $1` 转 orange 警示）
- `pendingApprovals > 0` 时头部出现橙色徽标，`onOpenApprovals` 负责跳转定位

## 代码演示

### 基础用法

四个 agent：一个编排中、一个已完成、一个失败（可重跑）；头部显示 1 待审批。

<DemoPreview name="AgentTeamBoardBasic" />

<<< ../demos/AgentTeamBoard/basic.tsx

## API

### AgentTeamBoardProps

| 属性               | 类型                   | 默认值         | 说明                                |
| ------------------ | ---------------------- | -------------- | ----------------------------------- |
| `agents`           | `AgentCardData[]`      | —              | 团队成员（同 AgentCard 的数据模型） |
| `title`            | `string`               | `'智能体团队'` | 面板标题                            |
| `selectedId`       | `string`               | —              | 当前选中的 agent id                 |
| `onSelectAgent`    | `(id: string) => void` | —              | 选中回调                            |
| `onRerunAgent`     | `(id: string) => void` | —              | 重跑回调（转发给失败卡）            |
| `pendingApprovals` | `number`               | `0`            | fleet 级待审批数                    |
| `onOpenApprovals`  | `() => void`           | —              | 点击待审批徽标的回调                |
| `className`        | `string`               | —              | 追加到根元素的类名                  |
