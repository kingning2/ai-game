# AttachmentList 附件列表

AI 附件缩略图条：圆角缩略图横向排列，悬停显示移除按钮，点击全屏预览（内置 Lightbox）。纯展示组件、无内边距，外边距与数据管理由调用方控制。

## 何时使用

- AI 输入框 / 消息气泡里展示**已选或已发送的图片附件**
- 需要移除能力时传入 `onRemove`，悬停出现 × 按钮，组件不持有删除逻辑
- 点击缩略图默认全屏预览（内置 Lightbox，自动锁滚动、Esc 关闭），不希望可点时传 `preview={false}`
- 列表为空时组件渲染 `null`，可以直接挂载无需条件判断

## 代码演示

### 基础用法

默认 64px 缩略图，点击任意一张全屏预览。

<DemoPreview name="AttachmentBasic" />

<<< ../demos/AttachmentList/basic.tsx

### 悬停移除

传入 `onRemove` 后启用移除按钮，悬停缩略图显示 ×，回调携带附件 `id`。

<DemoPreview name="AttachmentRemove" />

<<< ../demos/AttachmentList/remove.tsx

### 自定义尺寸

`size` 控制缩略图边长（px）：输入框内联用 40，默认 64，大图模式 96。

<DemoPreview name="AttachmentSize" />

<<< ../demos/AttachmentList/size.tsx

### 仅展示（关闭预览）

`preview={false}` 时缩略图不可点击，适合只读的消息历史。

<DemoPreview name="AttachmentNoPreview" />

<<< ../demos/AttachmentList/no-preview.tsx

## API

### AttachmentListProps

| 属性      | 说明                                          | 类型                   | 默认值 |
| --------- | --------------------------------------------- | ---------------------- | ------ |
| items     | 附件列表，为空数组时整体不渲染                | `AttachmentItem[]`     | -      |
| onRemove  | 传入后启用移除按钮，悬停显示 ×，回调附件 `id` | `(id: string) => void` | -      |
| size      | 缩略图边长（px）                              | `number`               | `64`   |
| preview   | 点击缩略图全屏预览（内置 Lightbox）           | `boolean`              | `true` |
| className | 附加类名，落在缩略图容器上                    | `string`               | -      |

### AttachmentItem

| 属性 | 说明                                                       | 类型     | 默认值 |
| ---- | ---------------------------------------------------------- | -------- | ------ |
| id   | 附件唯一标识，同时作为列表 key 与移除依据                  | `string` | -      |
| url  | 图片地址（本地用 `URL.createObjectURL(file)`，或远程 url） | `string` | -      |
| name | 文件名，用于替代文本、预览说明与移除按钮的 `aria-label`    | `string` | -      |

## 设计细节

- **内置 Lightbox**：点击预览直接复用 Lightbox 组件（portal 全屏、锁滚动、Esc 关闭），无需额外接线
- **移除按钮可见性**：悬停或键盘聚焦时淡入（`opacity-0 → 100`），半透明黑底白字保证任意图片上可读
- **object-cover 裁切**：缩略图统一 `object-cover` 填充方形格子，比例不一的图片也不会变形
- **空列表零渲染**：`items.length === 0` 时返回 `null`，输入框等容器里可无条件挂载
