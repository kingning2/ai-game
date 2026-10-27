# Form 表单

表单编排组件：基于 TanStack Form（headless）+ 组件库自有控件绑定。校验、提交、订阅状态全部由 TanStack `useForm` 承担，`TextField` / `SwitchField` / `CheckboxField` 负责把字段绑到自家输入控件上，`<Form>` 薄壳接管 `preventDefault` + `handleSubmit`。

## 何时使用

- 需要校验、错误提示、提交管理的多字段表单：注册、登录、设置页
- 错误提示需要与输入联动：错误时红色描边（`aria-invalid`）+ 底部红字文案
- 三类字段开箱即用：文本（`TextField`）、开关（`SwitchField`）、勾选（`CheckboxField`）
- 复杂表单逻辑（字段联动、异步校验）直接使用 re-export 的 `useForm`，不受组件封装限制
- 单字段、无提交语义的场景不必上 Form，直接用 `Input` + 受控状态即可

## 代码演示

### 基础用法

`useForm` 定义默认值与提交逻辑，`<Form>` 包裹，字段经 `form.Field` 渲染 prop 绑定到 `TextField`。

<DemoPreview name="FormBasic" />

<<< ../demos/Form/basic.tsx

### 校验与错误提示

`validators={{ onChange }}` 返回错误文案即触发：输入框红描边（`aria-invalid`）+ 底部红字，纠正后自动消失。

<DemoPreview name="FormValidation" />

<<< ../demos/Form/validation.tsx

### 三种字段类型

`TextField`（左 label + 输入 + hint）、`SwitchField`（左 label/hint，右开关）、`CheckboxField`（横向勾选）覆盖常见表单形态。

<DemoPreview name="FormFields" />

<<< ../demos/Form/fields.tsx

### 重置表单

`form.reset()` 回到 `defaultValues`，配合次级按钮使用。

<DemoPreview name="FormReset" />

<<< ../demos/Form/reset.tsx

### 异步提交

`onSubmit` 为异步函数：提交期间禁用按钮并显示进行中文案，完成后重置表单。

<DemoPreview name="FormAsyncSubmit" />

<<< ../demos/Form/async-submit.tsx

### 必选勾选

协议勾选用 `validators` 拦截：未勾选提交不通过，错误文案出现在勾选框下方。

<DemoPreview name="FormCheckboxRequired" />

<<< ../demos/Form/checkbox-required.tsx

### 综合示例

注册表单：两个校验文本字段 + 开关订阅 + 协议勾选 + 提交成功提示，`className` 给表单加卡片容器。

<DemoPreview name="FormFull" />

<<< ../demos/Form/full.tsx

## API

### FormProps

| 属性      | 说明                                                  | 类型                                                          | 默认值 |
| --------- | ----------------------------------------------------- | ------------------------------------------------------------- | ------ |
| form      | `useForm` 返回的表单实例（只要求具备 `handleSubmit`） | `FormLike`（`{ handleSubmit: () => Promise<void> \| void }`） | -      |
| className | 附加类名（常用于加卡片容器样式）                      | `string`                                                      | -      |
| children  | 表单内容，通常为若干 `form.Field`                     | `ReactNode`                                                   | -      |

### TextFieldProps

| 属性        | 说明                                   | 类型          | 默认值   |
| ----------- | -------------------------------------- | ------------- | -------- |
| field       | `form.Field` 渲染 prop 提供的字段实例  | `AnyFieldApi` | -        |
| label       | 字段标签（不传则不渲染）               | `string`      | -        |
| hint        | 底部提示文案（有错误时被错误文案替代） | `string`      | -        |
| placeholder | 占位文案                               | `string`      | -        |
| type        | 原生输入类型                           | `string`      | `'text'` |
| className   | 附加类名                               | `string`      | -        |

### SwitchFieldProps

| 属性      | 说明                                  | 类型          | 默认值 |
| --------- | ------------------------------------- | ------------- | ------ |
| field     | `form.Field` 渲染 prop 提供的字段实例 | `AnyFieldApi` | -      |
| label     | 开关左侧标题                          | `string`      | -      |
| hint      | 标题下方描述文案                      | `string`      | -      |
| className | 附加类名                              | `string`      | -      |

### CheckboxFieldProps

| 属性      | 说明                                  | 类型          | 默认值 |
| --------- | ------------------------------------- | ------------- | ------ |
| field     | `form.Field` 渲染 prop 提供的字段实例 | `AnyFieldApi` | -      |
| label     | 勾选框右侧标题                        | `string`      | -      |
| hint      | 标题下方描述文案                      | `string`      | -      |
| className | 附加类名                              | `string`      | -      |

### useForm

由 `@tanstack/react-form` re-export，签名与官方一致：

```ts
const form = useForm({
  defaultValues: { email: '' },
  onSubmit: async ({ value }) => {},
});
```

字段错误从 `field.state.meta.errors` 读取，取第一条 string（或 `{ message }`）作为可读文案。

## 设计细节

- **headless 分层**：校验/提交/订阅在 TanStack，布局与样式在字段绑定层，互不侵入
- **错误联动**：错误文案 `text-destructive` 同时给 `Input` 挂 `aria-invalid`，红描边由 Input 自身样式响应
- **hint 降级**：`FieldText` 优先显示错误，其次 hint，二者不同时出现，避免文案堆叠
- **字段绑定统一**：三个 Field 组件的 label / hint / error 布局规则一致，混排时视觉节奏统一
- **提交安全**：`<form noValidate>` 关闭浏览器原生校验，统一走 TanStack validators
