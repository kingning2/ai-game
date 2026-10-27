# Container 容器

页面内容容器：水平居中 + 响应式内边距，最大宽度走断点令牌而非写死 px，是页面级版心的统一入口。

## 何时使用

- 页面主体内容需要**水平居中**并限制最大阅读宽度时
- 希望全站页面左右留白统一（`px-4 sm:px-6` 响应式内边距）时
- 不同页面需要不同内容宽度档位（窄表单 / 常规文档 / 宽表格）时，用 `size` 三档区分
- 组件内部布局不要用 Container——它只负责页面级版心，内部排版交给 `Stack`

## 代码演示

### 基础用法

外层虚线框示意页面宽度：Container 始终水平居中，自带响应式左右内边距。

<DemoPreview name="ContainerBasic" />

<<< ../demos/Container/basic.tsx

### 三档宽度

`size` 提供三档最大宽度：`sm`（max-w-xl）适合窄表单，`md`（max-w-3xl）是默认文档宽度，`lg`（max-w-5xl）适合数据密集页面。

<DemoPreview name="ContainerSize" />

<<< ../demos/Container/size.tsx

### 页面骨架组合

Container 负责版心，内部交给 `SectionHeader` + `Stack` + `Divider` 组织内容——典型设置页骨架。

<DemoPreview name="ContainerPage" />

<<< ../demos/Container/page.tsx

## API

### ContainerProps

| 属性      | 说明                                                                        | 类型                   | 默认值 |
| --------- | --------------------------------------------------------------------------- | ---------------------- | ------ |
| size      | 内容最大宽度档位：`sm` → `max-w-xl`，`md` → `max-w-3xl`，`lg` → `max-w-5xl` | `'sm' \| 'md' \| 'lg'` | `'md'` |
| className | 附加类名，可覆盖内部样式                                                    | `string`               | -      |
| children  | 子元素                                                                      | `ReactNode`            | -      |

## 设计细节

- **宽度走断点令牌**：`max-w-xl / 3xl / 5xl` 是 Tailwind 断点令牌，而非写死 px，与全站栅格节奏一致
- **居中 + 内边距一体化**：`mx-auto w-full px-4 sm:px-6`，页面不再各自手写版心样式
- **只管版心不管排版**：内部一维布局交给 `Stack`，避免 Container 职责膨胀
