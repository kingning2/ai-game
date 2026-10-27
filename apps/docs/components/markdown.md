# Markdown 渲染

面向 AI 输出场景的流式 Markdown 渲染：react-markdown + remark-gfm，内置 Apple HIG 令牌默认主题，流式 fence 自动补闭合，全部元素可逐项覆盖。

## 何时使用

- 渲染大模型返回的 Markdown 回答（SSE 流式追加时渲染结果保持稳定）
- 需要安全的用户内容渲染：默认不执行原生 HTML，防 XSS
- 想让表格、代码块、引用块自带主题样式，又保留逐元素覆盖能力
- 不写 hook 的简单场景直接用 `<Markdown text={src} />` 薄壳

## 代码演示

### 基础用法

标题、粗斜体、列表、链接按内置主题渲染，明暗模式自适应。

<DemoPreview name="MarkdownBasic" />

<<< ../demos/Markdown/basic.tsx

### GFM 扩展

`remark-gfm` 开箱即用：表格（描边容器 + 斑马纹）、任务列表、删除线。

<DemoPreview name="MarkdownGfm" />

<<< ../demos/Markdown/gfm.tsx

### 代码展示

行内 code 与块级 fence 区分渲染：行内 tint 高亮胶囊，块级描边容器 + 横向滚动。

<DemoPreview name="MarkdownCode" />

<<< ../demos/Markdown/code.tsx

### 流式渲染

模拟 SSE 分片到达：fence 未闭合时 `stabilize` 自动补闭合（``` 与 ~~~ 分开计数），chunk 间渲染稳定不闪烁。

<DemoPreview name="MarkdownStreaming" />

<<< ../demos/Markdown/streaming.tsx

### 自定义元素

传 `options.components` 按 key 覆盖默认主题（链接变 tint 胶囊、引用块变卡片），调用方优先。

<DemoPreview name="MarkdownCustom" />

<<< ../demos/Markdown/custom.tsx

### 无样式模式

`unstyled: true` 跳过内置主题，完全使用传入的 components，适合嵌入已有设计系统。

<DemoPreview name="MarkdownUnstyled" />

<<< ../demos/Markdown/unstyled.tsx

## API

### Markdown 组件

| 属性    | 说明                                              | 类型                 | 默认值 |
| ------- | ------------------------------------------------- | -------------------- | ------ |
| text    | Markdown 源文本（流式场景传「已接收的累计文本」） | `string`             | -      |
| options | 渲染选项，见 UseMarkdownOptions                   | `UseMarkdownOptions` | -      |

### UseMarkdownOptions

| 属性       | 说明                                                 | 类型         | 默认值  |
| ---------- | ---------------------------------------------------- | ------------ | ------- |
| components | 覆盖/扩展元素渲染，与默认主题按 key 合并，调用方优先 | `Components` | -       |
| unstyled   | 跳过默认主题，完全使用传入的 components              | `boolean`    | `false` |
| stabilize  | 流式稳定化：为未闭合 fence 补闭合                    | `boolean`    | `true`  |
| className  | 附加到容器上的类名                                   | `string`     | -       |

### useMarkdown(text, options?)

把 markdown 文本渲染成 `ReactNode` 的 hook，参数与 `<Markdown />` 一致；适合需要把渲染结果嵌入自定义容器树的场景。

### stabilizeStreaming(src)

纯函数：SSE 流里 fence 未闭合时补上闭合标记（``` 与 ~~~ 分别按奇偶计数），返回补齐后的文本。默认由 `stabilize` 选项自动调用，也可在数据层单独使用。

### markdownComponents

内置默认主题的 `Components` 对象（h1–h6、p、a、strong、del、ul/ol/li、blockquote、hr、pre、code、table/th/td/tr、img），可单独 import 后自行合并复用。

## 设计细节

- **流式稳定化**：``` 与 ~~~ 围栏分开计数，只补未闭合的，chunk 间不会整块重排
- **安全默认**：默认不渲染原生 HTML（react-markdown 安全模式），链接统一 `target="_blank" rel="noreferrer"`
- **块/行内 code 区分**：依据 hast position 跨行或 `language-` class 判定，行内高亮、块级容器互不干扰
- **主题可剥离**：`unstyled` + `components` 组合可在保留流式能力的同时完全接管样式
