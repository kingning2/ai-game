# Progress 进度条

细线型进度条，用于展示任务、上传、加载等过程的完成比例。轨道与填充全部走设计令牌，宽度变化带生长感过渡。

## 何时使用

- 需要展示一个**有明确比例**的过程进度（上传、下载、导出、安装）
- 操作耗时不确定时改用 `Spinner`，进度可量化时才用 Progress
- 0–100 之外的取值会自动钳制，无需自己 `Math.min/Math.max`
- 进度变化频繁的场景直接更新 `value` 即可，组件自带 500ms 缓动，不要自己再包一层动画

## 代码演示

### 基础用法

传入 0–100 的 `value`，轨道填充宽度即百分比。下面依次为 0 / 25 / 60 / 100。

<DemoPreview name="ProgressBasic" />

<<< ../demos/Progress/basic.tsx

### 动态进度

`value` 是运行时数据，更新即触发 500ms ease-out 过渡，无需额外动画代码。

<DemoPreview name="ProgressDynamic" />

<<< ../demos/Progress/dynamic.tsx

### 超界自动钳制

`value` 小于 0 或大于 100 时自动收敛到边界，组件内部已做 clamp，业务层可放心传原始值。

<DemoPreview name="ProgressClamp" />

<<< ../demos/Progress/clamp.tsx

### 组合上传清单

进度条本身无内边距，配合 `Stack` 与文字即可拼出文件上传列表。

<DemoPreview name="ProgressUpload" />

<<< ../demos/Progress/upload.tsx

## API

### ProgressProps

| 属性      | 说明                            | 类型     | 默认值 |
| --------- | ------------------------------- | -------- | ------ |
| value     | 进度值，0–100，超出范围自动钳制 | `number` | `0`    |
| className | 附加类名，可覆盖内部样式        | `string` | -      |

### 无障碍

组件渲染 `role="progressbar"`，并输出 `aria-valuenow` / `aria-valuemin` / `aria-valuemax`，屏幕阅读器可直接播报进度。

## 设计细节

- **宽度即数据**：进度宽度通过 `style` 注入，其余样式全部走令牌，深浅色模式自动适配
- **生长感过渡**：宽度变化带 `duration-500 ease-out` 缓动，短任务也会有一次「长满」的动画
- **细线设计**：高度 6px（`h-1.5`）圆角胶囊，适合嵌入行内与卡片，不抢内容视觉焦点
