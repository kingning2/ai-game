# Slider 滑块

滑块用于在连续或离散区间内选取数值，交互规格对齐 iOS `UISlider`。组件库中唯一获取数值的输入件；纯展示进度请用 `Progress`，职责分离。

## 何时使用

- 需要拖动选择数值的场景：音量、亮度、不透明度、阈值调节
- 区间较小且可枚举时（如 0–100 按 10 步进），滑块比输入框更直觉
- 需要精确输入时与 `Input`（type=number）联动，拖动与键入双向同步
- 仅选取单个值；不支持双端区间选择
- 展示任务进度等只读场景不适用（用 `Progress`）

## 代码演示

### 基础用法

受控使用：`value` + `onChange` 由调用方管理，轨道左侧以 tint 填充表示已选区间。

<DemoPreview name="SliderBasic" />

<<< ../demos/Slider/basic.tsx

### 范围与步进

`min` / `max` / `step` 定义取值域：小数步进（0.1）与整数步进（10）均自动对齐档位，浮点误差已归一。

<DemoPreview name="SliderStep" />

<<< ../demos/Slider/step.tsx

### 带图标

配合图标表达语义（亮度由暗到明），数值实时回显。

<DemoPreview name="SliderWithIcons" />

<<< ../demos/Slider/with-icons.tsx

### 与输入框联动

Slider 与数字输入框共享同一状态：拖动即写入输入框，键入即更新滑块位置。

<DemoPreview name="SliderSynced" />

<<< ../demos/Slider/synced.tsx

## API

### SliderProps

| 属性       | 说明                               | 类型                      | 默认值 |
| ---------- | ---------------------------------- | ------------------------- | ------ |
| value      | 当前值（受控）                     | `number`                  | `0`    |
| min        | 最小值                             | `number`                  | `0`    |
| max        | 最大值                             | `number`                  | `100`  |
| step       | 步进（拖动与键盘方向键均按此对齐） | `number`                  | `1`    |
| onChange   | 值变化回调                         | `(value: number) => void` | -      |
| aria-label | 滑块的无障碍标签                   | `string`                  | -      |
| className  | 附加类名，可覆盖高度与宽度         | `string`                  | -      |

## 设计细节

- **细轨道规格**：4px 轨道 + 28px 白色圆形滑块；已选侧 tint 填充走逻辑属性（`start-0`），rtl 自动反向
- **拖出跟随**：`pointer capture` 捕获指针，拖出轨道后仍持续跟随，松开即停
- **弹簧放大**：按住滑块 spring 放大（`active:scale-110`），松手回弹
- **键盘操作**：方向键 ±step、`Home`/`End` 跳到边界；`touch-none` 防止拖动被页面滚动打断
- **无障碍语义**：`role="slider"` + `aria-valuemin/max/now`，请务必传 `aria-label`
