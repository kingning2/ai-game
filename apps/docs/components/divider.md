# Divider 分割线

发丝级分隔线，用半透明语义色（`--separator`）划分内容的逻辑边界，明暗主题下叠加在任何背景上都成立。

## 何时使用

- 区分内容**分组**或**区块**边界时，用水平分割线
- iOS 风格列表行与行之间，用 `inset` 缩进分隔线（跟随行内边距）
- 工具栏 / 操作行中需要竖向分隔并排元素时，用 `vertical`
- 纯装饰的留白需求不用 Divider，用 `Stack` 的 `gap` 表达间距

## 代码演示

### 基础用法

默认水平、占满整行，高度恒为 1px。

<DemoPreview name="DividerBasic" />

<<< ../demos/Divider/basic.tsx

### 缩进分隔线

`inset` 在行首缩进（`ms-4`），是 iOS 分组列表的标准样式，让分隔线与文字对齐。

<DemoPreview name="DividerInset" />

<<< ../demos/Divider/inset.tsx

### 垂直分隔线

`vertical` 切换为 1px 竖线并撑满父容器高度（`self-stretch`），适合并排内容的分界。

<DemoPreview name="DividerVertical" />

<<< ../demos/Divider/vertical.tsx

### 组合使用

配合 `SectionHeader` 与 `Stack` 组成 iOS 设置页分组列表：标题、卡片容器、缩进行分隔线。

<DemoPreview name="DividerCombine" />

<<< ../demos/Divider/combine.tsx

## API

### DividerProps

| 属性      | 说明                                                           | 类型      | 默认值  |
| --------- | -------------------------------------------------------------- | --------- | ------- |
| vertical  | 垂直分割线（`w-px self-stretch`），否则为水平（`h-px w-full`） | `boolean` | `false` |
| inset     | 列表内缩进分隔线（iOS 列表样式，跟随行内边距，`ms-4`）         | `boolean` | `false` |
| className | 附加类名，可覆盖内部样式                                       | `string`  | -       |

## 设计细节

- **发丝线材质**：颜色走 `--separator` 半透明语义色而非实色，同一根线在明暗主题、不同背景上都自然成立
- **无障碍语义**：渲染 `div[role="separator"]`，并随方向输出 `aria-orientation="horizontal" | "vertical"`
- **缩进跟随列表**：`inset` 用逻辑属性 `ms-4` 实现，RTL 布局下自动镜像
- **不承担间距职责**：分隔线只表达「边界」，行距请交给 `Stack` 的 `gap`，避免双重间距
