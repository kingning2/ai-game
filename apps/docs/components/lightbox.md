# Lightbox 灯箱

全屏图片预览：遮罩 + 居中大图。经 `createPortal` 挂到 `document.body`，不受祖先 `transform` / `filter` 影响，真正全屏；点击遮罩、`Esc` 或右上角关闭按钮均可退出。

## 何时使用

- 点击缩略图 / 附件图片后**全屏查看大图**
- 图片需要脱离页面布局层展示（页面有 `transform`、`overflow` 裁剪时尤其重要）
- 挂载期间自动锁定 body 滚动，关闭后恢复，无需业务处理
- 组件只负责展示与关闭，开关状态由父组件持有，方便与缩略图列表联动

## 代码演示

### 基础用法

父组件用 `open` state 控制挂载；点击遮罩 / 按 `Esc` / 点关闭按钮都会触发 `onClose`。

<DemoPreview name="LightboxBasic" />

<<< ../demos/Lightbox/basic.tsx

### 带说明文字

`caption` 显示在图片下方，常用于文件名或尺寸信息。

<DemoPreview name="LightboxCaption" />

<<< ../demos/Lightbox/caption.tsx

### 缩略图网格预览

多图场景：记录当前预览项，同一时间只挂载一个 Lightbox 实例。

<DemoPreview name="LightboxGallery" />

<<< ../demos/Lightbox/gallery.tsx

### 自定义遮罩样式

`className` 直接落在遮罩层，可加深底色、调整毛玻璃强度等。

<DemoPreview name="LightboxCustom" />

<<< ../demos/Lightbox/custom.tsx

### 受控开关

开关状态完全由外部驱动；Lightbox 内部的关闭交互（遮罩 / Esc / 按钮）通过 `onClose` 回传，父组件统一收敛状态。

<DemoPreview name="LightboxControlled" />

<<< ../demos/Lightbox/controlled.tsx

## API

### LightboxProps

| 属性      | 说明                                             | 类型         | 默认值 |
| --------- | ------------------------------------------------ | ------------ | ------ |
| src       | 图片地址                                         | `string`     | -      |
| alt       | 图片替代文本，同时作为对话框的 `aria-label`      | `string`     | -      |
| caption   | 图片下方说明文字（如文件名），不传则不渲染       | `string`     | -      |
| onClose   | 关闭回调：点击遮罩、按 `Esc`、点击关闭按钮时触发 | `() => void` | -      |
| className | 附加类名，落在全屏遮罩层，可覆盖底色 / 模糊      | `string`     | -      |

## 设计细节

- **原生 `<dialog>`**：`showModal()` 后 Esc 关闭、焦点圈定、`::backdrop` 顶层渲染全部由平台负责，组件内零键盘/焦点 JS
- **滚动锁纯 CSS**：`html:has(dialog:modal) { overflow: hidden }`，替代 JS 操控 `body.overflow`
- **进入动画**：`@starting-style` + `transition-behavior: allow-discrete`（Baseline 2024-08 全引擎），遮罩淡入 + 图片 0.96→1 放大，不用 keyframes
- **遮罩点击关闭**：事件目标是 dialog 元素本身（`e.target === dialog`），figure 内点击 `stopPropagation`
- **不受祖先影响**：top-layer 渲染，天然免portal、免 z-index 战争

- **Portal 全屏**：`createPortal` 挂 `document.body`，祖先的 `transform` / `filter` 不会截断 `fixed` 定位
- **滚动锁定**：挂载时 `body.overflow = 'hidden'`，卸载时恢复原值
- **进入动画**：遮罩 `fade-in`、图片 `zoom-in`，出现即有层次感
- **图片自适应**：`object-contain` + `h-[80vh]` + `w-[min(92vw,1600px)]`，横竖图都不裁切
- **无障碍**：`role="dialog"` + `aria-modal="true"`，关闭按钮有独立 `aria-label`
