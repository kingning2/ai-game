# Typewriter 打字机

流式打字机，专为 AI 回复展示设计：逐字输出、光标闪烁、支持跳过。16ms tick + 浮点累加器保证节奏均匀，长文本渲染只做字符串切片，几千字也不掉帧。

## 何时使用

- 展示 AI / 大模型的流式回复，模拟「正在生成」的阅读体验
- SSE 场景两种接法二选一：受控用法对同一 `text` state 追加后传入；非受控用法用 `ref.push(chunk)` 逐段喟入
- 需要给用户「跳过动画直接看全文」的出口时，配合 `ref.skip()` 使用
- 系统开启「减弱动态效果」时组件自动跳过逐字动画直接显示全文，无需业务处理

## 代码演示

### 基础用法

传入 `text`，组件从零逐字打出。默认速度 60 字/秒，接近流畅阅读的输出节奏。

<DemoPreview name="TypewriterBasic" />

<<< ../demos/Typewriter/basic.tsx

### 打字速度

`speed` 是每秒字符数：重要结论用慢速强调，长回复用高速追赶进度。

<DemoPreview name="TypewriterSpeed" />

<<< ../demos/Typewriter/speed.tsx

### 光标行为

`cursor` 控制是否显示光标（`text` 为空时光标持续闪烁 = 等待首 token）；`hideCursorOnDone` 控制打完后是否隐藏，多轮对话可传 `false` 让光标常驻结尾。

<DemoPreview name="TypewriterCursor" />

<<< ../demos/Typewriter/cursor.tsx

### 非受控流式推送

不传 `text`，用 `ref.push(chunk)` 模拟 SSE 逐段喟入；`ref.skip()` 立即显示全部已接收文本（「跳过」按钮）。两种用法同时存在时以 `text` 为准。

<DemoPreview name="TypewriterStream" />

<<< ../demos/Typewriter/stream.tsx

### 完成回调

`onFinished` 在全部文本显示完毕时触发一次；流式追加后重新打完会再次触发。

<DemoPreview name="TypewriterFinished" />

<<< ../demos/Typewriter/finished.tsx

## API

### TypewriterProps

| 属性             | 说明                                                                                                   | 类型         | 默认值 |
| ---------------- | ------------------------------------------------------------------------------------------------------ | ------------ | ------ |
| text             | 目标文本（受控用法）。流式追加时对同一 state 追加后传入，组件从上次进度续打；不传则走非受控 `ref.push` | `string`     | -      |
| speed            | 打字速度：每秒字符数                                                                                   | `number`     | `60`   |
| cursor           | 显示打字光标；text 为空时光标持续闪烁（等待首 token）                                                  | `boolean`    | `true` |
| hideCursorOnDone | 打完后隐藏光标；传 `false` 让光标常驻结尾                                                              | `boolean`    | `true` |
| onFinished       | 已接收文本全部显示完毕时触发；流式追加后再次打完会再次触发                                             | `() => void` | -      |
| className        | 附加类名，可覆盖内部样式                                                                               | `string`     | -      |

### TypewriterHandle（ref 方法）

| 方法  | 说明                                                                | 类型                      |
| ----- | ------------------------------------------------------------------- | ------------------------- |
| skip  | 跳过动画：立即显示全部已接收文本（配合「跳过」按钮）                | `() => void`              |
| push  | 命令式喟入一个 chunk（非受控用法，在 SSE `onmessage` 回调里直接调） | `(chunk: string) => void` |
| reset | 清空缓冲、进度归零（非受控用法的重置）                              | `() => void`              |

## 设计细节

- **均匀节奏**：16ms tick + 浮点累加器，`speed` 不足 1 字/tick 时均摊推进，不会一顿一顿
- **渲染即切片**：没有逐字 span，只是字符串 `slice(0, count)`，长回复也不掉帧
- **跳过也安全**：`skip` 用 `Number.MAX_SAFE_INTEGER` + 渲染 clamp，批处理下「跳过 + 新文本」同时 setState 也正确
- **无障碍**：`aria-label` 读全文、可见切片 `aria-hidden`，屏幕阅读器不会逐字播报；`prefers-reduced-motion` 直接显示全文
- **重发新文本**：换 `key` 重挂载即可；流式追加只增不减时无需处理
