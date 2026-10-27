# SectionHeader 区块标题

iOS 分组列表的 section header 规格：小号加粗、带字距的次要文字色标题，是「耐看的底层秩序」的一部分。

## 何时使用

- iOS 风格**分组列表**每组内容的标题（设置页的「通用」「隐私」）
- **表单分组**前的小节标题，与分组卡片保持 2 档间距
- 需要与大标题（页面 `h1`）区分的**次级小节**；本组件固定渲染 `h2`，请勿再手动加字号
- 纯视觉强调请用 `Stack` 间距或 `Divider`，不要拿标题当分隔符用

## 代码演示

### 基础用法

固定 iOS 规格：13px / semibold / 字距 `+0.06em` / 次要文字色；英文自动大写（GENERAL）。

<DemoPreview name="SectionHeaderBasic" />

<<< ../demos/SectionHeader/basic.tsx

### 分组列表

配合 `Stack` + 卡片容器 + `Divider`（inset）组成 iOS 设置页的多分组列表。

<DemoPreview name="SectionHeaderList" />

<<< ../demos/SectionHeader/list.tsx

### 表单分组

与 `HStack`（`justify="between"`）组合做键值对表单行，标题 + 分组卡片是标准搭档。

<DemoPreview name="SectionHeaderForm" />

<<< ../demos/SectionHeader/form.tsx

## API

### SectionHeaderProps

继承原生 `h2` 元素全部属性（`HTMLAttributes<HTMLHeadingElement>`）：

| 属性      | 说明                                                      | 类型                                 | 默认值 |
| --------- | --------------------------------------------------------- | ------------------------------------ | ------ |
| children  | 标题内容，英文自动 `uppercase`                            | `ReactNode`                          | -      |
| className | 附加类名，可覆盖内部样式                                  | `string`                             | -      |
| ...rest   | 其余原生 h2 属性（`id`、`onClick`、`style`、`aria-*` 等） | `HTMLAttributes<HTMLHeadingElement>` | -      |

## 设计细节

- **固定 iOS 规格**：13px / `font-semibold` / `tracking-[0.06em]` / `text-secondary-label`，不提供 size 变体——规格的唯一性就是秩序感
- **中文同样受益**：中文没有大小写，但字距与次要文字色同样生效，中英文混排节奏一致
- **语义正确**：渲染 `h2`，配合 `id` 可直接作为锚点，文档大纲自动收录
