# Card 卡片

卡片是内容的容器单元 —— Apple 分组单元格样式：在 grouped 背景上承载一组相关信息，圆角、分隔与投影全部走令牌。

## 何时使用

- 将一组相关信息（标题、描述、数据、操作）聚合为一个视觉单元
- 在 `grouped` 灰底页面上区分内容分组（设置页、详情页的分区）
- 概览页并排展示多张信息卡（统计、宫格、套餐选择）
- 卡片本身可点击时，用 `className` 补充 hover / 选中态

## 代码演示

### 基础用法

标题 + 描述的最简结构，内边距 `p-4`，圆角与投影内置。

<DemoPreview name="CardBasic" />

<<< ../demos/Card/basic.tsx

### 组合头部信息

卡片是纯容器，内部内容用 `HStack` / `VStack` 自由组合 —— 这里放了 Avatar、名称、徽标与简介。

<DemoPreview name="CardProfile" />

<<< ../demos/Card/profile.tsx

### 统计卡片

多张等宽卡片横向排布，适合概览页的关键指标。

<DemoPreview name="CardStat" />

<<< ../demos/Card/stat.tsx

### 设置宫格

二维宫格布局属于 grid 场景（flex 只做一维布局），配合卡片形成 iOS 设置入口样式。

<DemoPreview name="CardGrid" />

<<< ../demos/Card/grid.tsx

### 可点击选中

通过 `className` 叠加 `ring` 选中态与 `hover:bg-tertiary-fill`，选中状态由组件外部 `useState` 管理。

<DemoPreview name="CardInteractive" />

<<< ../demos/Card/interactive.tsx

## API

### CardProps

| 属性      | 说明                                                     | 类型                             | 默认值 |
| --------- | -------------------------------------------------------- | -------------------------------- | ------ |
| className | 附加类名，可覆盖内部样式（如 `p-6`、`ring-2 ring-tint`） | `string`                         | -      |
| ...rest   | 其余原生 div 属性（`onClick`、`style`、`aria-*` 等）     | `HTMLAttributes<HTMLDivElement>` | -      |

## 设计细节

- **顶部内高光**：`shadow-[inset_0_1px_0_0_rgb(255_255_255/0.06)]` 在深色模式下提亮顶缘，是 Apple 深色卡片「受光」的关键细节
- **分组背景体系**：底色用 `bg-secondary-grouped-background`，放在 `grouped` 灰底上才成立；投影走 `shadow-card` 令牌
- **纯容器设计**：不内置标题 / 头部 / 底栏插槽，内容布局交给 `Stack` 系列原语，避免插槽 API 膨胀
- **圆角阶梯**：`rounded-xl` 与 List、DataTable 等展示类组件对齐
