# WebSearch 联网搜索

展示 Agent 联网搜索的完整过程：多角度查询（queries[]）、每条查询独立经历 `searching → done`，并产出 AI 合成答案与来源引用列表。数据模型对齐 pi v1.2.0 的 `web_search` 工具调用结构。纯展示组件，零业务逻辑。

## 何时使用

- Agent 对话流中回显一次「联网搜索」工具调用：搜了什么（多角度 query）、正在搜还是搜完、答案是什么、引用了哪些来源
- 需要来源可点击时传入 `onOpenSource`；不传则来源渲染为纯文本行
- 组件零时序假设：`queries` 数组由调用方逐条 push / 原位更新，即可驱动整个卡片（检索中 Spinner、流式答案、来源回填）

## 代码演示

### 基础用法

三条查询：两条已完成（含合成答案 + 来源引用），一条仍在检索中；头部显示进度与总耗时。

<DemoPreview name="WebSearchBasic" />

<<< ../demos/WebSearch/basic.tsx

### 推进交互

模拟真实搜索过程：点击「发起搜索」逐条 push 查询 → 1s 后开始流式追加答案 → 完成后回填来源与耗时；再点追加第二条查询。

<DemoPreview name="WebSearchLive" />

<<< ../demos/WebSearch/live.tsx

## API

### WebSearchProps

| 属性           | 类型                    | 默认值       | 说明                                                     |
| -------------- | ----------------------- | ------------ | -------------------------------------------------------- |
| `queries`      | `WebSearchQuery[]`      | —            | 查询数组（多角度），逐条 push / 原位更新即可驱动整个卡片 |
| `provider`     | `string`                | —            | 提供方徽标文字，如 `'openai'` / `'brave'`                |
| `title`        | `string`                | `'联网搜索'` | 卡片标题                                                 |
| `onOpenSource` | `(url: string) => void` | —            | 来源点击回调；不传则渲染为纯文本行                       |
| `className`    | `string`                | —            | 追加到根元素的类名                                       |

### WebSearchQuery

| 属性         | 类型                | 说明                             |
| ------------ | ------------------- | -------------------------------- |
| `query`      | `string`            | 查询词                           |
| `status`     | `'searching' \\     | 'done'`                          | 单条查询状态 |
| `answer`     | `string?`           | AI 合成答案（可流式追加）        |
| `sources`    | `WebSearchSource[]` | 来源引用列表                     |
| `durationMs` | `number?`           | 该条查询耗时，结束后由调用方回填 |

### WebSearchSource

| 属性      | 类型      | 说明                 |
| --------- | --------- | -------------------- |
| `title`   | `string`  | 来源标题             |
| `url`     | `string`  | 来源链接             |
| `snippet` | `string?` | 摘要片段（两行截断） |
