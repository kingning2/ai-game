# Spinner 加载指示

轻量环形加载指示器，SVG 描边旋转实现。颜色跟随文字色，适合嵌入按钮、列表、卡片等各种等待场景。

## 何时使用

- 操作耗时**不可预估**（无明确比例）时表达等待，有比例时改用 `Progress`
- 按钮触发异步请求后，用 `Spinner` + `disabled` 组合表达加载中（按钮不内置 loading 属性）
- 列表、卡片等区块首屏拉取数据时的区块级占位
- 需要换色时通过 `className` 传入文字色即可，指示器用 `currentColor` 描边

## 代码演示

### 基础用法

三档尺寸：`sm`（16px）用于按钮内联，`md`（20px）是默认尺寸，`lg`（24px）用于区块级加载。

<DemoPreview name="SpinnerBasic" />

<<< ../demos/Spinner/basic.tsx

### 自定义颜色

指示器用 `currentColor` 描边，传入任意语义文字色即可整体换色。

<DemoPreview name="SpinnerColor" />

<<< ../demos/Spinner/color.tsx

### 按钮加载中

遵循 Apple HIG 的组合式用法：按钮置灰禁用，内嵌小号 Spinner 表达处理中。

<DemoPreview name="SpinnerInline" />

<<< ../demos/Spinner/inline.tsx

### 列表加载占位

Spinner 配合弱色文案是最轻量的加载占位，无需骨架屏。

<DemoPreview name="SpinnerPlaceholder" />

<<< ../demos/Spinner/placeholder.tsx

## API

### SpinnerProps

| 属性      | 说明                                        | 类型                   | 默认值 |
| --------- | ------------------------------------------- | ---------------------- | ------ |
| size      | 尺寸档位：`sm` 16px / `md` 20px / `lg` 24px | `'sm' \| 'md' \| 'lg'` | `'md'` |
| className | 附加类名，可用于覆盖颜色（传入文字色类）    | `string`               | -      |

### 无障碍

组件渲染 `role="status"` 并自带 `aria-label="加载中"`，屏幕阅读器会播报加载状态。

## 设计细节

- **颜色跟随文字**：轨道与弧线都是 `currentColor`，放进不同文字色的容器自动适配，无需双色配置
- **半透明轨道**：底圈 20% 透明度、前弧全色，旋转时层次清晰，不会与背景糊在一起
- **圆头描边**：弧线端点 `strokeLinecap="round"`，旋转过渡更顺滑
