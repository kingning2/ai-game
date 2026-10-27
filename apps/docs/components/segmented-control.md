# SegmentedControl 分段控制

分段控件用于在 2–5 个互斥的平级选项间切换，交互规格对齐 iOS `UISegmentedControl`（滑动 thumb 版）。

## 何时使用

- 视图切换：列表 / 卡片 / 网格、日 / 周 / 月统计周期
- 平级筛选器：排序方式、内容分类，切换即时生效
- 选项是「模式」而非「开关」时优先用它；只有两项且互为开关用 `Switch`
- 选项超过 5 个时改用 `Select`，避免分段挤压
- 需要承载滑动动画的 iOS 风格控件；普通选项跳变样式可用一组按钮替代

## 代码演示

### 基础用法

受控使用：`value` + `onChange` 由调用方管理，泛型约束 `T extends string` 保证取值安全。

<DemoPreview name="SegmentedBasic" />

<<< ../demos/SegmentedControl/basic.tsx

### 切换内容视图

选中态驱动下方内容联动渲染，是最常见的视图切换场景。

<DemoPreview name="SegmentedContent" />

<<< ../demos/SegmentedControl/content.tsx

### 撑满容器

传 `className="w-full"` 让分段等分容器宽度，适合放在页面顶部做全宽筛选。

<DemoPreview name="SegmentedFullWidth" />

<<< ../demos/SegmentedControl/full-width.tsx

### 作为筛选器

排序切换联动列表内容，演示筛选器完整交互。

<DemoPreview name="SegmentedFilter" />

<<< ../demos/SegmentedControl/filter.tsx

## API

### SegmentedControlProps

| 属性       | 说明                           | 类型                                | 默认值 |
| ---------- | ------------------------------ | ----------------------------------- | ------ |
| options    | 分段选项（等宽排列）           | `ReadonlyArray<SegmentedOption<T>>` | -      |
| value      | 当前选中值（受控）             | `T`                                 | -      |
| onChange   | 选中回调                       | `(value: T) => void`                | -      |
| aria-label | 控件的无障碍标签               | `string`                            | -      |
| className  | 附加类名，如 `w-full` 撑满容器 | `string`                            | -      |

> 泛型 `T extends string`；`value` 取未知值时选中索引回退到第一段（`Math.max(0, ...)`）。

### SegmentedOption

| 属性  | 说明               | 类型                   | 默认值 |
| ----- | ------------------ | ---------------------- | ------ |
| value | 分段唯一值         | `T`（`string` 子类型） | -      |
| label | 分段文案（纯文本） | `string`               | -      |

## 设计细节

- **滑动 thumb**：选中态是绝对定位的白底 thumb，随选择 spring 滑动（过冲回弹），而不是每段各自切背景的跳变
- **等宽分段**：每段 `flex-1` 等分；thumb 宽度按 `(100% - 4px) / 段数` 计算
- **方向自适应**：thumb 位移走 CSS 变量 + rtl 变体，rtl 布局下自动反向；文字颜色仍即时切换
- **无障碍语义**：容器 `role="radiogroup"`，每段 `role="radio"` + `aria-checked`
- **按压反馈**：按下即时 `scale-0.97`，200ms 恢复
