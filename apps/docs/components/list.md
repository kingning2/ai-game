# List 列表

分组列表容器 + 列表行，复刻 iOS 分组列表（inset grouped）规格：发丝线分隔、分组单元格底色、可点击行带按压反馈。

## 何时使用

- 设置页的分组选项列表
- 需要以「行」为单位展示实体（成员、文件、会话），行首有头像 / 图标、行尾有状态或控件
- 需要点击整行触发操作（导航、选择、删除）
- 表单之外的信息罗列与摘要展示

## 代码演示

### 基础用法

`List` 提供分组底色与行间发丝线，`ListItem` 只需 `title`。

<DemoPreview name="ListBasic" />

<<< ../demos/List/basic.tsx

### 行首媒体区

`media` 插槽放图标或头像，行布局为 `media | 内容弹性区 | trailing 固定区`。

<DemoPreview name="ListMedia" />

<<< ../demos/List/media.tsx

### 行尾信息与徽标

`trailing` 放值文本或 Badge，次级信息写进 `subtitle`（超长自动截断）。

<DemoPreview name="ListTrailing" />

<<< ../demos/List/trailing.tsx

### 可点击行

传入 `onClick` 时行渲染为 `button`，内置 hover 高亮与按压缩放；选中或操作结果由外部状态管理。

<DemoPreview name="ListClickable" />

<<< ../demos/List/clickable.tsx

### 设置页组合

`trailing` 放 `Switch` 控件，行本身不点击 —— 这是 iOS 设置页的标准布局。

<DemoPreview name="ListSettings" />

<<< ../demos/List/settings.tsx

## API

### ListProps

| 属性      | 说明                             | 类型        | 默认值 |
| --------- | -------------------------------- | ----------- | ------ |
| className | 附加类名，可覆盖圆角、最大宽度等 | `string`    | -      |
| children  | 列表行，通常为一组 `ListItem`    | `ReactNode` | -      |

### ListItemProps

| 属性      | 说明                                             | 类型         | 默认值 |
| --------- | ------------------------------------------------ | ------------ | ------ |
| media     | 行首媒体区（头像 / 图标）                        | `ReactNode`  | -      |
| title     | 行标题（超长截断）                               | `ReactNode`  | -      |
| subtitle  | 行副标题，`text-xs` 次要文字（超长截断）         | `ReactNode`  | -      |
| trailing  | 行尾区（值文本 / 徽标 / 控件）                   | `ReactNode`  | -      |
| onClick   | 点击回调；传入后行渲染为 `button` 并启用按压反馈 | `() => void` | -      |
| className | 附加类名，可覆盖行内边距等                       | `string`     | -      |

## 设计细节

- **发丝线分隔**：`divide-y divide-separator`，首尾行的边线被 `overflow-hidden` 圆角裁掉，与 iOS 分组列表一致
- **一维 flex 行**：媒体区不缩放、内容区 `min-w-0` 可截断、trailing 区 `shrink-0`，长文案不会把行尾控件挤出
- **可点击即按钮**：`onClick` 存在时行渲染为 `button` 元素，键盘可达；未传时保持 `div` 语义
- **按压反馈**：可点击行 hover 高亮 `bg-quaternary-fill`、按下 `scale-[0.98]`，遵循平台按压手感
