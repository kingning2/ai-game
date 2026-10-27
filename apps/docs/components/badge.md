# Badge 徽标

徽标用于标注对象的状态、数量或分类。胶囊外形 + 系统色 10% 透明度底（Apple 着色徽章做法），轻量不抢焦点，状态语义一目了然。

## 何时使用

- 需要标注某个对象的状态（进行中、已完成、已失败等）
- 列表、卡片、表格中需要轻量的分类或标签
- 提示内容的时效性标记（如「新」「Beta」「推荐」）
- 按语义区分优先级：成功用 `green`、警告用 `orange`、危险用 `red`、中性用 `gray`
- 信息强调程度低于标题或按钮，只是一行小字的状态说明时

## 代码演示

### 基础用法

默认 `gray` 中性灰徽标，适合无强语义的普通标签。

<DemoPreview name="BadgeBasic" />

<<< ../demos/Badge/basic.tsx

### 六种语义色调

`tone` 提供 blue / tint / green / orange / red / gray 六种系统色，底色为同色 10% 透明度，深浅色模式自动适配。

<DemoPreview name="BadgeTone" />

<<< ../demos/Badge/tone.tsx

### 搭配内容使用

徽标常出现在列表行的行尾表示状态，或卡片标题旁作为版本 / 阶段标记。

<DemoPreview name="BadgeStatus" />

<<< ../demos/Badge/status.tsx

### 自定义样式

通过 `className` 覆盖尺寸、圆角、描边等；其余原生 `span` 属性（`title`、`onClick`、`aria-*` 等）直接透传。

<DemoPreview name="BadgeCustom" />

<<< ../demos/Badge/custom.tsx

## API

### BadgeProps

| 属性      | 说明                                                  | 类型                                                         | 默认值   |
| --------- | ----------------------------------------------------- | ------------------------------------------------------------ | -------- |
| tone      | 语义色调，决定底色与文字色                            | `'blue' \| 'tint' \| 'green' \| 'orange' \| 'red' \| 'gray'` | `'gray'` |
| className | 附加类名，可覆盖内部样式                              | `string`                                                     | -        |
| ...rest   | 其余原生 span 属性（`title`、`onClick`、`aria-*` 等） | `HTMLAttributes<HTMLSpanElement>`                            | -        |

### BadgeTone

| 值       | 语义   | 典型场景         |
| -------- | ------ | ---------------- |
| `blue`   | 信息蓝 | 进行中、提示     |
| `tint`   | 主题色 | 推荐、品牌向标签 |
| `green`  | 成功绿 | 已完成、已通过   |
| `orange` | 警告橙 | 待处理、审核中   |
| `red`    | 危险红 | 已失败、已阻塞   |
| `gray`   | 中性灰 | 草稿、默认态     |

## 设计细节

- **胶囊造型**：`rounded-full` 全圆角，`px-2 py-0.5 text-xs font-medium`，不依赖边框
- **10% 透明度底**：彩色文字 + 同色 10% 底，与 Apple 着色徽章一致，深浅色模式无需两套色值
- **不折行**：内置 `whitespace-nowrap`，文案过长时应截短文字而不是换行
