# TaskPlan 任务计划

Agent 多步任务的可视化主干：步骤清单 + 四态状态（pending/running/done/failed）+ 进度统计头。

## 何时使用

- Agent 执行多步任务时展示计划与实时进度 —— ChatBot 与 Agent 的分水岭组件
- 步骤状态完全由调用方维护（与 ToolExecution 同约定），组件零内部状态、零时序假设
- running 步骤用 Spinner 高亮，done 弱化为次要文字 —— 视觉焦点永远在"正在做什么"

## 代码演示

### 四态步骤

done / running / pending / failed + 进度头自动统计。

<DemoPreview name="TaskPlanBasic" />

<<< ../demos/TaskPlan/basic.tsx

### 推进交互

点按钮推进 running 流转；全完成可重置。

<DemoPreview name="TaskPlanAdvance" />

<<< ../demos/TaskPlan/advance.tsx

## API

### TaskStep

| 属性   | 说明                      | 类型                                           |
| ------ | ------------------------- | ---------------------------------------------- |
| title  | 步骤标题                  | `string`                                       |
| detail | 补充说明（工具名/耗时等） | `string`                                       |
| status | 步骤状态                  | `'pending' \| 'running' \| 'done' \| 'failed'` |

### TaskPlanProps

| 属性      | 说明     | 类型         | 默认值       |
| --------- | -------- | ------------ | ------------ |
| title     | 计划标题 | `string`     | `'任务计划'` |
| steps     | 步骤数组 | `TaskStep[]` | -            |
| className | 附加类名 | `string`     | -            |

## 设计细节

- **语义色沿用 ToolExecution 体系**：done=`green/15`、failed=`red/15`、running=tint Spinner、pending=描边空心
- **进度头**：`已完成 {doneCount}/{total} 步`，`tabular-nums` 防跳动
- **running 提示**：有 running 步骤时底部显示"正在执行当前步骤…"横条
