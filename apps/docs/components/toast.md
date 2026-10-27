# Toast 轻通知

全局轻通知：后台任务完成/失败的 iOS 横幅式提示。`ToastProvider` 包裹应用根，内部用 `useToast()` 触发。

## 何时使用

- Agent 后台任务（导出/部署/长请求）完成或失败时通知
- 不打断对话流：横幅从顶部滑入、点击即关、默认 3.2s 自动消失
- 同屏最多 3 条，超出移除最旧

## 代码演示

<DemoPreview name="ToastBasic" />

<<< ../demos/Toast/basic.tsx

## API

### useToast()

返回 `{ toast, dismiss }`：

| 方法    | 说明         | 签名                              |
| ------- | ------------ | --------------------------------- |
| toast   | 弹一条通知   | `(options: ToastOptions) => void` |
| dismiss | 手动关闭一条 | `(id: number) => void`            |

### ToastOptions

| 属性        | 说明           | 类型                             | 默认值   |
| ----------- | -------------- | -------------------------------- | -------- |
| title       | 通知标题       | `string`                         | -        |
| description | 补充说明       | `string`                         | -        |
| kind        | 通知类别       | `'info' \| 'success' \| 'error'` | `'info'` |
| duration    | 自动关闭毫秒数 | `number`                         | `3200`   |

## 设计细节

- **材质模糊**：`bg-material + backdrop-blur-xl`（Apple 系统材质），悬停微降透明度
- **两段动画**：入场 `slide-in-top`（抽屉曲线），退场先 `fade-out` 再卸载 —— 退场不硬切
- **语义图标**：success=green 对勾圆、error=red 叉圆、info=tint 圆点
- **可访问性**：容器 `aria-live="polite"`，屏幕阅读器会播报新通知
