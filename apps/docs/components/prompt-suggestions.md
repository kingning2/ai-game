# PromptSuggestions 快捷提示

快捷提示词：空状态或输入区上方的建议 chips，点击把 prompt 交回调用方（通常填入 Composer）。

## 何时使用

- 新会话空状态引导用户"能问什么"
- 输入区上方给上下文相关的快捷追问
- 默认胶囊式（wrap 换行）；`stacked` 切列表式（带箭头），适合引导页大按钮

## 代码演示

<DemoPreview name="PromptSuggestionsBasic" />

<<< ../demos/PromptSuggestions/basic.tsx

## API

### PromptSuggestionsProps

| 属性     | 说明                     | 类型                       | 默认值  |
| -------- | ------------------------ | -------------------------- | ------- |
| prompts  | 提示词数组               | `string[]`                 | -       |
| onSelect | 点击回调                 | `(prompt: string) => void` | -       |
| title    | 标题，如 `'试试这些'`    | `string`                   | -       |
| stacked  | 列表式（带箭头）替代胶囊 | `boolean`                  | `false` |

## 设计细节

- **stagger 入场**：`animate-rise` + 每项 40ms 延迟（封顶 200ms），`prefers-reduced-motion` 下自动清零
- **胶囊**：`bg-secondary-grouped-background shadow-card`，hover 转 `tertiary-fill`
- **纯展示**：组件不管理输入框状态，onSelect 后做什么由调用方决定
