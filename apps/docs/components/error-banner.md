# ErrorBanner 错误横幅

请求失败的重试横幅：`kind` 直接复用 `@agent-packages/shared` 的 `ApiErrorKind`，默认文案按类别区分 —— 上层只管把 error 传下来。

## 何时使用

- 请求失败后在对话流/页面上方提示，可重试、可关闭
- kind 与 `ApiError.kind` 一一对应：`network`（网络/DNS/CORS）、`business`（业务 code 非成功）、`http`（网关层错误）、`parse`（响应格式异常）、`abort`（取消/超时）
- 自定义文案时传 `title` 覆盖默认；`message` 放原始错误信息

## 代码演示

<DemoPreview name="ErrorBannerKinds" />

<<< ../demos/ErrorBanner/kinds.tsx

## API

### ErrorBannerProps

| 属性      | 说明                             | 类型           | 默认值 |
| --------- | -------------------------------- | -------------- | ------ |
| title     | 错误标题；不传按 kind 取默认文案 | `string`       | -      |
| message   | 错误详情（原始 message）         | `string`       | -      |
| kind      | 错误类别，决定默认文案           | `ApiErrorKind` | -      |
| onRetry   | 重试回调（传入即显示重试按钮）   | `() => void`   | -      |
| onDismiss | 关闭回调（传入即显示关闭按钮）   | `() => void`   | -      |

## 设计细节

- **默认文案映射**：network→"网络连接失败"、business→"请求被服务端拒绝"、http→"服务暂时不可用"、parse→"响应格式异常"、abort→"请求已取消"
- **样式**：`bg-destructive/10` + `border-destructive/20`，警示图标 + `role="alert"`
- **重试按钮**：plain 变体覆盖 destructive 文字色，与横幅融为一体
