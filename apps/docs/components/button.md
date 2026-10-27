# Button 按钮

按钮用于开始一个即时操作。遵循 Apple 规范：语义令牌配色、spring 按压反馈、毛玻璃光泽渐变。

## 何时使用

- 标记一个（或封装一组）操作命令，响应用户点击触发业务逻辑
- 主操作用 `variant="solid"`（tint 实底），次级用 `tinted` / `plain`，工具栏用 `gray`，删除等危险操作用 `destructive`
- 需要禁用态时传原生 `disabled`，自动降透明度并禁用指针事件
- 三档高度（sm 36 / md 40 / lg 48）覆盖表单、工具栏、大屏 CTA 场景

## 代码演示

### 基础用法

默认 `solid` 变体 + `md` 尺寸，悬停浮起 1px 带阴影，按下 spring 缩放回弹。

<DemoPreview name="ButtonBasic" />

<<< ../demos/Button/basic.tsx

### 五种视觉变体

`solid` 实底 / `tinted` 淡底 / `plain` 纯文字 / `gray` 灰底 / `destructive` 危险红。彩色填充上一律白字，dark 模式也不反转。

<DemoPreview name="ButtonVariant" />

<<< ../demos/Button/variant.tsx

### 三种尺寸

`sm` 36px / `md` 40px / `lg` 48px，高度、间距、字号全部走令牌阶梯。

<DemoPreview name="ButtonSize" />

<<< ../demos/Button/size.tsx

### 图标按钮

`leading` 场景直接在 children 里内联 SVG 图标，`gap` 自动控制图标与文字间距。

<DemoPreview name="ButtonIcon" />

<<< ../demos/Button/icon.tsx

### 加载中

loading 态由业务侧控制（如提交中禁用按钮 + Spinner 占位）。

<DemoPreview name="ButtonLoading" />

<<< ../demos/Button/loading.tsx

### 禁用状态

原生 `disabled`：透明度 40%、禁用指针事件、不参与悬停浮起。

<DemoPreview name="ButtonDisabled" />

<<< ../demos/Button/disabled.tsx

## API

### ButtonProps

继承原生 `ButtonHTMLAttributes<HTMLButtonElement>`（`onClick`、`disabled`、`type`、`aria-*` 等全部透传）。

| 属性      | 说明                                                                  | 类型                              | 默认值     |
| --------- | --------------------------------------------------------------------- | --------------------------------- | ---------- |
| variant   | 视觉变体：`'solid' \| 'tinted' \| 'plain' \| 'gray' \| 'destructive'` | `ButtonVariant`                   | `'solid'`  |
| size      | 尺寸：`'sm' \| 'md' \| 'lg'`                                          | `ButtonSize`                      | `'md'`     |
| type      | 原生按钮类型                                                          | `'button' \| 'submit' \| 'reset'` | `'button'` |
| className | 附加类名，可覆盖内部样式                                              | `string`                          | -          |
| children  | 按钮内容（文字、图标或混合）                                          | `ReactNode`                       | -          |

### 设计细节

- **spring 按压**：`ease-spring` 曲线贯穿全程，按下即时 `scale(0.96)`，松开带过冲回弹——iOS 手感核心
- **光泽渐变**：彩色实底上叠 `from-white/20 via-white/5 to-transparent` 上下渐变，告别平面感
- **悬停浮起**：solid / gray / destructive 悬停 `-translate-y-px` + `shadow-card`，次级变体不浮起
- **令牌配色**：`tint` / `fill` / `destructive` 语义令牌，明暗模式自动切换
