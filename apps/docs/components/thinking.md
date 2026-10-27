# Thinking 思考过程

AI 思维链面板（DeepSeek R1 / Claude thinking 风格）：标题行可折叠展开，思考中显示脉动「思考中…」，结束显示「已深度思考（用时 N 秒）」；内容区左侧竖线 + 弱色小字，流式追加自动滚到底。

## 何时使用

- 展示大模型的思维链 / 推理过程，让「等待」可感知、可回看
- 思考中（`thinking=true`）自动展开并跟随流式内容滚动；结束后自动收起，默认不打扰正文
- 用户随时可点击标题回看完整思维链，无需额外交互代码
- 需要与外部布局联动（外层折叠按钮、多面板同步）时改用 `open` 受控模式

## 代码演示

### 流式思考中

完整生命周期模拟：思考中逐段追加内容（自动展开、滚到底），结束后自动收起并显示用时。

<DemoPreview name="ThinkingStreaming" />

<<< ../demos/Thinking/streaming.tsx

### 结束收起

`thinking=false` 时标题变为「已深度思考（用时 N 秒）」，面板默认收起。

<DemoPreview name="ThinkingCollapsed" />

<<< ../demos/Thinking/collapsed.tsx

### 点击回看

非受控模式下点击标题即可展开 / 收起；`onToggle` 在每次切换时回调最新状态。

<DemoPreview name="ThinkingReview" />

<<< ../demos/Thinking/review.tsx

### 受控模式

传入 `open` 后组件进入受控模式：自动展开 / 收起失效，展开状态完全由外部驱动，`onToggle` 只做通知。

<DemoPreview name="ThinkingControlled" />

<<< ../demos/Thinking/controlled.tsx

## API

### ThinkingBlockProps

| 属性      | 说明                                                                           | 类型                      | 默认值         |
| --------- | ------------------------------------------------------------------------------ | ------------------------- | -------------- |
| content   | 思维链内容（可流式追加，保留换行）                                             | `string`                  | -              |
| thinking  | 思考进行中；为 `true` 时标题显示脉动「思考中…」                                | `boolean`                 | `false`        |
| duration  | 思考用时（秒）；thinking 结束后标题附带「（用时 N 秒）」                       | `number`                  | -              |
| label     | 结束态标题文案                                                                 | `string`                  | `'已深度思考'` |
| open      | 手动展开 / 收起覆盖自动行为；thinking 重新开始时重置（非受控模式）。传入即受控 | `boolean`                 | -              |
| onToggle  | 展开 / 收起切换回调，参数为切换后的状态                                        | `(open: boolean) => void` | -              |
| className | 附加类名，可覆盖内部样式                                                       | `string`                  | -              |

## 设计细节

- **自动展开收起**：思考中自动展开（`override ?? thinking`），结束自动收起；用户手动点开后以手动状态优先，thinking 重新开始时手动覆盖自动重置
- **grid-rows 折叠动画**：`0fr → 1fr` 过渡实现高度自适应的顺滑折叠，无需测量内容高度
- **流式跟滚**：思考中内容追加时自动 `scrollTop = scrollHeight`，结束后停止跟随，方便回看
- **内容限高**：正文区 `max-h-64` 内部滚动，超长思维链不会撑爆页面
