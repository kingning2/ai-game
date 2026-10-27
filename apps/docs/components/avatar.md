# Avatar 头像

头像用于代表一个人或一个实体。圆形，优先渲染图片，无图时回退到 tint 底色缩写。

## 何时使用

- 用户、成员、联系人等实体的身份标识（列表行、卡片头部、评论）
- 团队成员列表、协作者展示
- 需要按尺寸分级的头像场景（工具栏 `xs`、列表行 `sm`、个人页 `lg`）
- 无图片资源时用姓名首字母 / 末字作占位

## 代码演示

### 基础用法

不传 `src` 时渲染 `fallback` 缩写 —— 一般取姓名首字母或中文姓名末字。

<DemoPreview name="AvatarBasic" />

<<< ../demos/Avatar/basic.tsx

### 四档尺寸

`xs`(24px) / `sm`(32px) / `md`(40px，默认) / `lg`(48px)，字号随尺寸同步缩放。

<DemoPreview name="AvatarSize" />

<<< ../demos/Avatar/size.tsx

### 图片头像

传入 `src` 渲染图片（`object-cover` 裁剪），`alt` 提供无障碍描述；第四个未传 `src`，展示 `fallback` 回退。

<DemoPreview name="AvatarImage" />

<<< ../demos/Avatar/image.tsx

### 自定义样式

通过 `className` 覆盖圆角、底色、文字色或叠加描边。

<DemoPreview name="AvatarCustom" />

<<< ../demos/Avatar/custom.tsx

### 搭配列表

`media` 插槽放 `sm` 尺寸头像，是成员列表最常见的组合。

<DemoPreview name="AvatarList" />

<<< ../demos/Avatar/list.tsx

## API

### AvatarProps

| 属性      | 说明                                          | 类型                              | 默认值 |
| --------- | --------------------------------------------- | --------------------------------- | ------ |
| src       | 图片地址；存在时渲染图片，否则渲染 `fallback` | `string`                          | -      |
| alt       | 图片的无障碍描述                              | `string`                          | `''`   |
| fallback  | 无图片时的回退缩写（一般取姓名首字母 / 末字） | `string`                          | -      |
| size      | 头像尺寸                                      | `'xs' \| 'sm' \| 'md' \| 'lg'`    | `'md'` |
| className | 附加类名，可覆盖圆角、底色等                  | `string`                          | -      |
| ...rest   | 其余原生 span 属性（`title`、`aria-*` 等）    | `HTMLAttributes<HTMLSpanElement>` | -      |

## 设计细节

- **tint 回退底**：缩写态为 `bg-tint/10` + `text-tint`，与品牌主题呼应，多头像并排时风格统一
- **图片裁剪**：`overflow-hidden rounded-full` + `object-cover`，任意比例的图片都裁成正圆
- **不内置加载失败回退**：`src` 存在即渲染 `<img>`；需要失败兜底时由调用方控制 `src` 的传值
- **shrink-0**：在 flex 行内不被压缩，列表行媒体区永不变形
