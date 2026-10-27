# Composer AI 输入域

聊天式输入区（Apple HIG 风格），为 AI 对话场景组装的多行输入组件：附件条、模型选择、发送 / 停止一体。附件与模型状态由调用方管理（受控组件）。

## 何时使用

- AI 聊天 / 对话应用的底部输入区，需要多行文本 + 附件 + 模型选择的组合
- 需要流式生成状态：生成中发送按钮切换为停止按钮（`streaming` + `onStop`）
- 支持三种图片上传入口：按钮选择、拖拽、粘贴（`onAddFiles` 统一回调）
- 中文输入法组词中的 Enter 不触发发送，Shift + Enter 换行
- 普通表单的单行输入不要用它，用 `Input` / `TextField`

## 代码演示

### 基础用法

受控使用：`value` + `onChange` 管理文本，`onSend` 在点发送或按 Enter（有内容时）触发。

<DemoPreview name="ComposerBasic" />

<<< ../demos/Composer/basic.tsx

### 流式生成与停止

`streaming` 时发送按钮切换为停止按钮，`onStop` 接管停止逻辑。

<DemoPreview name="ComposerStreaming" />

<<< ../demos/Composer/streaming.tsx

### 模型选择

传 `models` 后左下角出现 ghost 选择器（向上弹出），`model` + `onModelChange` 受控。

<DemoPreview name="ComposerModels" />

<<< ../demos/Composer/models.tsx

### 附件与移除

`attachments` 在输入框上方渲染缩略图条，`onRemoveAttachment` 移除；示例图片为内联 SVG data URI。

<DemoPreview name="ComposerAttachments" />

<<< ../demos/Composer/attachments.tsx

### 自动增高

`maxRows` 限制自动增高的行数封顶，超出后内部滚动；`placeholder` 自定义占位文案。

<DemoPreview name="ComposerMaxRows" />

<<< ../demos/Composer/max-rows.tsx

### 禁用状态

禁用时整块降透明度，输入、模型选择、图片与发送全部不可用。

<DemoPreview name="ComposerDisabled" />

<<< ../demos/Composer/disabled.tsx

### 完整聊天示例

组合模型选择、发送、模拟流式回复与停止的消息列表，贴近真实用法。

<DemoPreview name="ComposerChat" />

<<< ../demos/Composer/chat.tsx

## API

### ComposerProps

| 属性               | 说明                                                   | 类型                      | 默认值        |
| ------------------ | ------------------------------------------------------ | ------------------------- | ------------- |
| value              | 输入文本（受控）                                       | `string`                  | -             |
| onChange           | 文本变化回调                                           | `(value: string) => void` | -             |
| onSend             | 发送回调（Enter 或点发送；需有文本或附件才可发）       | `() => void`              | -             |
| onStop             | 停止生成回调（streaming 时生效）                       | `() => void`              | -             |
| streaming          | 生成中：发送按钮切换为停止按钮                         | `boolean`                 | `false`       |
| disabled           | 是否禁用整块输入区                                     | `boolean`                 | `false`       |
| placeholder        | 占位文案                                               | `string`                  | `'输入消息…'` |
| models             | 可选模型列表（传入后显示选择器）                       | `ComposerModel[]`         | -             |
| model              | 当前模型 id（受控）                                    | `string`                  | -             |
| onModelChange      | 模型切换回调                                           | `(id: string) => void`    | -             |
| attachments        | 附件列表（受控，渲染为缩略图条）                       | `ComposerAttachment[]`    | `[]`          |
| onAddFiles         | 追加文件回调（已过滤为图片），来源：按钮 / 拖拽 / 粘贴 | `(files: File[]) => void` | -             |
| onRemoveAttachment | 移除附件回调                                           | `(id: string) => void`    | -             |
| maxRows            | 自动增高行数上限，超出内部滚动                         | `number`                  | `8`           |
| className          | 附加类名                                               | `string`                  | -             |

### ComposerModel

| 属性  | 说明                         | 类型     | 默认值 |
| ----- | ---------------------------- | -------- | ------ |
| id    | 模型唯一 id                  | `string` | -      |
| name  | 模型名（选择器文案）         | `string` | -      |
| badge | 可选标注，如「默认」「推理」 | `string` | -      |

### ComposerAttachment

与 `AttachmentItem` 结构一致。

| 属性 | 说明                                                       | 类型     | 默认值 |
| ---- | ---------------------------------------------------------- | -------- | ------ |
| id   | 附件唯一 id                                                | `string` | -      |
| url  | 图片地址（本地用 `URL.createObjectURL(file)`，或远程 url） | `string` | -      |
| name | 可选文件名                                                 | `string` | -      |

## 设计细节

- **组合而非重造**：附件条复用 `AttachmentList`，模型选择复用 `Select`（ghost + 向上弹出），样式与行为全站一致
- **自动增高**：优先纯 CSS `field-sizing: content`（Chrome 123+ / Firefox 135+ 等现代浏览器零 JS），不支持的旧浏览器自动降级为 JS 按行高重算；`maxRows` 用 `calc(n * lh + padding)` 封顶，超出后内部滚动
- **发送门槛**：`canSend` 要求非禁用、非生成中，且有文本或附件；不满足时发送按钮 30% 透明度不可点
- **IME 安全**：中文组词中的 Enter（`isComposing`）不触发发送，Shift + Enter 换行
- **容器反馈**：容器聚焦时描边变 tint + 光环（与 Input 一致），拖拽悬停时同样高亮提示可放置
