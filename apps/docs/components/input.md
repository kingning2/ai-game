# Input 输入框

文本输入框是最基础的表单控件，用于接收单行文本。组件为描边式（C 端 web 主流预期）：常态白底 + 发丝描边，聚焦后描边变主题色并晕开光环。

## 何时使用

- 需要用户输入单行文本：昵称、邮箱、搜索词、验证码等
- 与 `Form` / `TextField` 搭配，作为标准表单字段的底层输入件
- 敏感信息（密码、密钥）使用 `type="password"` 遮盖显示
- 校验失败时挂 `aria-invalid`，以红色描边与光环提示错误，而非弹窗打断
- 仅用于输入；纯数值选择的场景优先用 `Slider` 或 `SegmentedControl`

## 代码演示

### 基础用法

受控使用：`value` + `onChange` 由调用方管理，输入即回显。

<DemoPreview name="InputBasic" />

<<< ../demos/Input/basic.tsx

### 输入类型

透传原生 `type`：密码遮盖、搜索框（带清除按钮）、数字调步，均由浏览器原生能力支持。

<DemoPreview name="InputTypes" />

<<< ../demos/Input/types.tsx

### 校验错误

`aria-invalid` 一挂即生效：静默时淡红描边提醒，聚焦后切换为红色描边 + 红色光环。

<DemoPreview name="InputError" />

<<< ../demos/Input/error.tsx

### 禁用状态

禁用时整体降为 40% 透明度并移除指针事件，原生 `disabled` 语义保留。

<DemoPreview name="InputDisabled" />

<<< ../demos/Input/disabled.tsx

## API

### InputProps

| 属性      | 说明                                                                                                     | 类型                                    | 默认值   |
| --------- | -------------------------------------------------------------------------------------------------------- | --------------------------------------- | -------- |
| type      | 原生输入类型                                                                                             | `string`                                | `'text'` |
| className | 附加类名，可覆盖内部样式                                                                                 | `string`                                | -        |
| ...rest   | 其余原生 input 属性（`value`、`onChange`、`placeholder`、`disabled`、`aria-invalid`、`autoComplete` 等） | `InputHTMLAttributes<HTMLInputElement>` | -        |

## 设计细节

- **描边分层**：与卡片同底色，靠发丝 `separator` 描边区分层级，页面更干净
- **聚焦反馈**：一次 200ms 过渡完成「描边变 tint + 3px 光环 + 卡片阴影浮起」，白底不加深
- **错误三态**：静默淡红描边 → 聚焦红描边 → 红色光环，全程不额外加线，避免抖动
- **高度令牌**：固定 `h-9` + `rounded-lg` + `text-sm`，与 Select、Composer 输入区对齐
