# ToolExecution 工具执行

Agent 工具调用的可视化卡片：状态图标（待执行圆点 / Spinner / 绿勾 / 红叉）+ 等宽工具名 + 耗时，头部整行可点披露参数与输出；失败态红字输出块并显示重试按钮。

## 何时使用

- 展示 Agent 回复中的工具调用过程（web_search / read_file / bash …），让「执行中」可感知、结果可回看
- **纯状态驱动**：组件只根据 `status` 字段渲染，不做任何时序假设 —— running→success/error 的流转完全由调用方控制，SSE / 轮询 / 模拟数据都能接
- 参数与输出默认折叠，点头部展开（`grid-rows 0fr→1fr` 过渡，高度自适应无魔法数字）
- 失败时配合 `onRetry` 显示重试按钮，重试逻辑（重跑同一调用、换参重试）由调用方决定

## 代码演示

### 成功生命周期

点击运行模拟一次调用：running（Spinner）→ success，回填输出与耗时。

<DemoPreview name="ToolExecutionBasic" />

<<< ../demos/ToolExecution/basic.tsx

### 多工具序列与失败重试

Agent 一次回复中的工具调用流（最新在前）；失败项点击重试后恢复成功。

<DemoPreview name="ToolExecutionSequence" />

<<< ../demos/ToolExecution/sequence.tsx

## API

### ToolExecutionProps

| 属性        | 说明                                 | 类型                | 默认值  |
| ----------- | ------------------------------------ | ------------------- | ------- |
| execution   | 工具执行数据（见 ToolExecutionData） | `ToolExecutionData` | -       |
| defaultOpen | 非受控初始是否展开参数与输出         | `boolean`           | `false` |
| onRetry     | 传入后失败态显示重试按钮，点击时回调 | `() => void`        | -       |
| className   | 附加类名，可覆盖内部样式             | `string`            | -       |

### ToolExecutionData

| 字段       | 说明                                  | 类型                                             | 默认值 |
| ---------- | ------------------------------------- | ------------------------------------------------ | ------ |
| id         | 唯一标识（列表渲染 key）              | `string`                                         | -      |
| name       | 工具名，如 `web_search` / `bash`      | `string`                                         | -      |
| args       | 任意可序列化入参，JSON.stringify 展示 | `unknown`                                        | -      |
| status     | 执行状态，图标随状态切换              | `'pending' \| 'running' \| 'success' \| 'error'` | -      |
| result     | 成功输出（文本，保留换行）            | `string`                                         | -      |
| error      | 失败信息，红字输出块展示              | `string`                                         | -      |
| durationMs | 耗时，结束后由调用方回填              | `number`                                         | -      |

## 设计细节

- **状态图标**：`pending` 空心圆点 / `running` Spinner（sm）/ `success` 绿勾（`bg-green/15` 圆底）/ `error` 红叉（`bg-red/10` 圆底），颜色全部走 HIG 系统色令牌
- **披露动画**：内容区用 `grid-rows 0fr ↔ 1fr` 过渡 —— 唯一能做到「高度自适应 + 无魔法数字」的纯 CSS 方案；箭头旋转 200ms ease-out
- **参数块**：等宽字体 + `bg-tertiary-fill`，`JSON.stringify(args, null, 2)` 格式化；输出块同样式，错误块 `bg-red/10 text-red`
- **输出限高**：`max-h-64` 内部滚动，超长输出不会撑爆页面；耗时数字 `tabular-nums` 防跳动
- **状态驱动**：组件不持有任何执行逻辑，重试时调用方把 `status` 改回 `running` 并清空 `error` / `durationMs` 即可复用同一卡片
