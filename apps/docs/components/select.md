# Select 选择器

下拉选择（受控），用于在互斥选项中单选。提供 outline（表单描边控件）与 ghost（轻量文字按钮）两种形态，浮层菜单支持向上 / 向下弹出。

## 何时使用

- 选项数量在 2–10 个之间的互斥单选；超过时优先用输入 + 搜索方案
- 表单字段使用 `outline`，与 `Input` 保持一致的圆角与聚焦光环
- 工具行内的轻量切换（如 Composer 的模型选择）使用 `ghost`
- 浮层下方空间不足时（贴近视口底部）改用 `placement="top"` 向上弹出
- 选项需要附加说明（如「默认」「推理」标注）时使用 `badge`

## 代码演示

### 基础用法

受控使用：`value` 为 `string | null`（`null` 时显示 placeholder），点选后通过 `onChange` 回传。点外部或 Esc 关闭浮层。

<DemoPreview name="SelectBasic" />

<<< ../demos/Select/basic.tsx

### 标注与禁用项

`badge` 渲染为浅色小字标注；`disabled` 项在菜单中置灰不可选中。

<DemoPreview name="SelectBadge" />

<<< ../demos/Select/badge.tsx

### ghost 变体与向上弹出

`variant="ghost"` 去掉描边、只保留文字与悬停底色，适合嵌在工具行中；`placement="top"` 让菜单向上弹出。

<DemoPreview name="SelectGhost" />

<<< ../demos/Select/ghost.tsx

### 禁用状态

禁用时整体降为 60% 透明度并移除指针事件，浮层不再展开。

<DemoPreview name="SelectDisabled" />

<<< ../demos/Select/disabled.tsx

## API

### SelectProps

| 属性        | 说明                          | 类型                      | 默认值      |
| ----------- | ----------------------------- | ------------------------- | ----------- |
| value       | 当前选中值，`null` 表示未选择 | `string \| null`          | -           |
| onChange    | 选中回调                      | `(value: string) => void` | -           |
| options     | 选项列表                      | `SelectOption[]`          | -           |
| placeholder | 未选择时的占位文案            | `string`                  | `'请选择'`  |
| disabled    | 是否禁用                      | `boolean`                 | `false`     |
| variant     | 形态：表单描边 / 轻量文字按钮 | `'outline' \| 'ghost'`    | `'outline'` |
| placement   | 浮层弹出方向                  | `'bottom' \| 'top'`       | `'bottom'`  |
| className   | 附加类名                      | `string`                  | -           |
| ariaLabel   | 触发按钮与菜单的无障碍标签    | `string`                  | -           |

### SelectOption

| 属性     | 说明                                     | 类型        | 默认值 |
| -------- | ---------------------------------------- | ----------- | ------ |
| value    | 选项唯一值                               | `string`    | -      |
| label    | 选项文案，可渲染任意节点                 | `ReactNode` | -      |
| badge    | 可选标注（浅色小字），如「默认」「推理」 | `ReactNode` | -      |
| disabled | 该项是否禁用                             | `boolean`   | -      |

## 设计细节

- **无障碍语义**：触发按钮挂 `aria-haspopup="listbox"` + `aria-expanded`，菜单为 `role="listbox"`、选项为 `role="option"` + `aria-selected`
- **关闭交互**：浮层打开时监听 `pointerdown`（点外部）与 `Escape`，自动清理监听器
- **选中反馈**：当前项加粗变 tint 色，右侧对勾图标；badge 在触发按钮与菜单中都会回显
- **宽度策略**：outline 占满容器宽度（`w-full`），菜单 `min-w-full` 不小于触发器；ghost 收缩为内容宽度
