# Switch 开关

开关用于在两个互斥状态间切换（开 / 关），交互规格对齐 iOS `UISwitch`。

## 何时使用

- 即时生效的设置项开关：通知、同步、勿扰模式等（无需「保存」按钮）
- 设置列表行内使用：左侧 label + 描述文案，右侧开关
- 需要「提交才生效」的字段请改用 `Checkbox`，语义更明确
- 表单场景优先用 `Form` 的 `SwitchField`，自带 label / hint / 错误文案布局

## 代码演示

### 基础用法

受控使用：`checked` + `onCheckedChange` 由调用方管理，点击即回调新状态。

<DemoPreview name="SwitchBasic" />

<<< ../demos/Switch/basic.tsx

### 设置行

iOS 设置风格的经典布局：左侧标题与描述、右侧开关，卡片内纵向排列。

<DemoPreview name="SwitchSettingRow" />

<<< ../demos/Switch/setting-row.tsx

### 开关分组

多个设置项归入一张卡片，统一用对象状态管理。

<DemoPreview name="SwitchGroup" />

<<< ../demos/Switch/group.tsx

### 禁用状态

禁用时降为 40% 透明度并移除指针事件，开启 / 关闭两种禁用态样式一致。

<DemoPreview name="SwitchDisabled" />

<<< ../demos/Switch/disabled.tsx

## API

### SwitchProps

| 属性            | 说明                                                                                 | 类型                                      | 默认值  |
| --------------- | ------------------------------------------------------------------------------------ | ----------------------------------------- | ------- |
| checked         | 是否开启                                                                             | `boolean`                                 | `false` |
| onCheckedChange | 切换回调，参数为切换后的状态                                                         | `(checked: boolean) => void`              | -       |
| className       | 附加类名，可覆盖轨道尺寸与配色                                                       | `string`                                  | -       |
| ...rest         | 其余原生 button 属性（`disabled`、`aria-label`、`id` 等；`onChange`/`value` 已剔除） | `ButtonHTMLAttributes<HTMLButtonElement>` | -       |

## 设计细节

- **iOS 规格**：轨道 `w-11`（44px）/ 滑块 `size-5`（20px）/ 边距 2px，位移 20px 正好滑满
- **开启态配色**：使用 systemGreen（Apple 默认开关色），关闭态为中性 fill 语义色
- **spring 回弹**：滑块位移走 300ms spring 曲线，带过冲回弹；按下时轨道降透明度反馈
- **方向自适应**：滑块位移走逻辑属性，rtl 布局下自动反向
- **无障碍语义**：`role="switch"` + `aria-checked`，建议传入 `aria-label` 或配合 label 使用
