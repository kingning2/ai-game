# NavBar 导航栏

页面顶部的导航栏，iOS 毛玻璃材质的 web 形态：半透明材质 + 背景模糊，滚动内容从栏底透出。

## 何时使用

- 页面顶部需要固定标题栏，并在滚动时保持可见（默认 `sticky` 吸顶）
- 左侧放返回箭头 / Logo、中间放标题、右侧放动作按钮的三段式布局
- 需要滚动时内容从栏底**模糊透出**的 iOS 质感时（默认毛玻璃材质）
- 卡片、弹层等局部区域需要一栏标题时，传 `sticky={false}` 作为普通静态栏使用

## 代码演示

### 基础用法

只传 `title` 的最简形态。示例传了 `sticky={false}` 让示例在文档页中保持静态；默认吸顶，见「滚动毛玻璃」场景。

<DemoPreview name="NavBarBasic" />

<<< ../demos/NavBar/basic.tsx

### 带返回箭头

`leading` 放左侧区域（返回箭头、Logo），建议为纯图标按钮提供 `aria-label`。

<DemoPreview name="NavBarLeading" />

<<< ../demos/NavBar/leading.tsx

### 带操作区

`trailing` 放右侧动作区，可以并排放多个文字按钮或图标按钮。

<DemoPreview name="NavBarTrailing" />

<<< ../demos/NavBar/trailing.tsx

### 标题居中

标题占据中间弹性区（`flex-1`），只有两侧内容**等宽**时标题才视觉居中——用等宽占位让 `leading` 与 `trailing` 平衡。

<DemoPreview name="NavBarCentered" />

<<< ../demos/NavBar/centered.tsx

### 滚动毛玻璃

默认 `sticky` 吸顶 + 毛玻璃材质（`bg-material` + `backdrop-blur-xl` + `backdrop-saturate-150`），滚动内容从栏底模糊透出。示例限定在局部滚动容器内演示。

<DemoPreview name="NavBarScroll" />

<<< ../demos/NavBar/scroll.tsx

### 静态栏（非吸顶）

`sticky={false}` 关闭吸顶，导航栏作为普通静态栏参与文档流，适合嵌入卡片、弹层头部。

<DemoPreview name="NavBarStatic" />

<<< ../demos/NavBar/static.tsx

## API

### NavBarProps

| 属性      | 说明                                                          | 类型                          | 默认值 |
| --------- | ------------------------------------------------------------- | ----------------------------- | ------ |
| leading   | 左侧区域（如返回箭头、Logo）                                  | `ReactNode`                   | -      |
| title     | 中间标题，超出一行自动截断（`truncate`）；未传时渲染 children | `ReactNode`                   | -      |
| trailing  | 右侧动作区（如主题切换、编辑按钮）                            | `ReactNode`                   | -      |
| sticky    | 吸顶（`sticky top-0 z-40`）；关闭时作为普通静态栏使用         | `boolean`                     | `true` |
| className | 附加类名，可覆盖内部样式                                      | `string`                      | -      |
| children  | 未传 `title` 时作为中间内容渲染                               | `ReactNode`                   | -      |
| ...rest   | 其余原生 header 元素属性（`id`、`aria-*` 等）                 | `HTMLAttributes<HTMLElement>` | -      |

## 设计细节

- **毛玻璃材质**：`bg-material`（半透明令牌，随明暗切换）+ `backdrop-blur-xl` + `backdrop-saturate-150`，滚动内容从栏底模糊透出，即 iOS 控制中心质感
- **优雅降级**：`prefers-reduced-transparency` 时 `--material` 自动变实底、模糊同步去除，由令牌层完成，组件无感知
- **发丝线收边**：底部 `border-b border-separator` 与内容区轻分隔，延续 iOS 列表语言
- **排版内聚**：内部固定 `h-14` 高度与三段式弹性布局，内容宽度对齐 `Container` 由外部决定，组件只管排版
