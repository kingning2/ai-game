# Checkbox 复选框

复选框用于在一组互不排斥的选项中多选，或表示单项确认（如协议勾选）。

## 何时使用

- 多选场景：筛选条件、权限勾选、批量选择列表项
- 单项确认但需「提交后生效」：勾选协议后才能点击注册按钮
- 与 `Switch` 的区别：`Switch` 用于即时生效的开关，`Checkbox` 用于需要提交确认的选择
- 表单场景优先用 `Form` 的 `CheckboxField`，自带 label / hint / 错误文案布局

## 代码演示

### 基础用法

受控使用：`checked` + `onCheckedChange` 由调用方管理。

<DemoPreview name="CheckboxBasic" />

<<< ../demos/Checkbox/basic.tsx

### 多选组

用数组管理多选状态，点选即增删；label 与复选框组合提升点击热区。

<DemoPreview name="CheckboxGroup" />

<<< ../demos/Checkbox/group.tsx

### 协议勾选

勾选前禁用提交按钮，是「确认后生效」的典型交互。

<DemoPreview name="CheckboxAgreement" />

<<< ../demos/Checkbox/agreement.tsx

### 禁用状态

禁用时降为 40% 透明度并移除指针事件，勾选态样式保留。

<DemoPreview name="CheckboxDisabled" />

<<< ../demos/Checkbox/disabled.tsx

## API

### CheckboxProps

| 属性            | 说明                                                                                 | 类型                                      | 默认值  |
| --------------- | ------------------------------------------------------------------------------------ | ----------------------------------------- | ------- |
| checked         | 是否勾选                                                                             | `boolean`                                 | `false` |
| onCheckedChange | 切换回调，参数为切换后的状态                                                         | `(checked: boolean) => void`              | -       |
| className       | 附加类名，可覆盖尺寸与配色                                                           | `string`                                  | -       |
| ...rest         | 其余原生 button 属性（`disabled`、`aria-label`、`id` 等；`onChange`/`value` 已剔除） | `ButtonHTMLAttributes<HTMLButtonElement>` | -       |

## 设计细节

- **勾选态配色**：tint 底 + 白色对勾，未勾选为浅灰底 + 发丝描边
- **按压反馈**：`pointer-down` 即时 `scale-90` 缩小，150ms 快速恢复，手感干脆
- **无障碍语义**：`role="checkbox"` + `aria-checked`；单独使用时建议传 `aria-label` 或包在 `label` 里
- **尺寸**：固定 `size-5`（20px）+ `rounded-md`，与文本基线对齐时可加 `mt-0.5`
