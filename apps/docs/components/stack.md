# Stack 层叠容器

弹性布局原语，按一维方向（横 / 纵）排列子元素。间距、对齐、分布全部映射到设计令牌阶梯，是页面排版的默认选择。

## 何时使用

- 页面内的一维排列（纵向堆叠卡片、横向排一行按钮）一律用 Stack，`grid` 只留给二维场景
- 需要统一、有节奏的**令牌间距**时，用 `gap` 阶梯取值，禁止写死任意 px
- 工具栏 / 操作行需要把某个元素推到行尾时，用 `HStack` + `Spacer` 组合
- 横向排列、纵向排列都需要语义化别名时，直接用 `HStack` / `VStack`，比 `direction` 更一目了然

## 代码演示

### 基础用法

默认纵向排列，`gap` 控制子元素间距（默认 2）。

<DemoPreview name="StackBasic" />

<<< ../demos/Stack/basic.tsx

### 排列方向

`direction` 切换横 / 纵；`HStack` / `VStack` 是等价的语义化简写。

<DemoPreview name="StackDirection" />

<<< ../demos/Stack/direction.tsx

### 间距阶梯

`gap` 只接受设计令牌阶梯（0 / 0.5 / 1 / 1.5 / 2 / 3 / 4 / 5 / 6 / 8 / 10 / 12），保证全站间距节奏一致。

<DemoPreview name="StackGap" />

<<< ../demos/Stack/gap.tsx

### 对齐与分布

`align` 控制交叉轴对齐，`justify` 控制主轴分布；子元素高度不同时对齐效果最直观。

<DemoPreview name="StackAlign" />

<<< ../demos/Stack/align.tsx

### 组合使用

`HStack` + `Spacer` 把操作推到行尾，`VStack` 纵向组织内容区，`Divider` 分隔——典型工具栏排版。

<DemoPreview name="StackCombine" />

<<< ../demos/Stack/combine.tsx

## API

### StackProps

| 属性      | 说明                                 | 类型                                                                | 默认值      |
| --------- | ------------------------------------ | ------------------------------------------------------------------- | ----------- |
| direction | 排列方向                             | `'row' \| 'column'`                                                 | `'column'`  |
| gap       | 子元素间距（令牌阶梯，禁止任意数值） | `0 \| 0.5 \| 1 \| 1.5 \| 2 \| 3 \| 4 \| 5 \| 6 \| 8 \| 10 \| 12`    | `2`         |
| align     | 交叉轴对齐方式                       | `'stretch' \| 'start' \| 'center' \| 'end' \| 'baseline'`           | `'stretch'` |
| justify   | 主轴分布方式                         | `'start' \| 'center' \| 'end' \| 'between' \| 'around' \| 'evenly'` | `'start'`   |
| wrap      | 是否允许换行                         | `boolean`                                                           | `false`     |
| as        | 渲染的元素标签                       | `ElementType`                                                       | `'div'`     |
| className | 附加类名，可覆盖内部样式             | `string`                                                            | -           |
| style     | 内联样式                             | `CSSProperties`                                                     | -           |
| children  | 子元素                               | `ReactNode`                                                         | -           |

### 附带导出

| 组件   | 说明                                       | 类型                                                    |
| ------ | ------------------------------------------ | ------------------------------------------------------- |
| HStack | 横向堆叠（`direction` 固定为 `'row'`）     | `(props: Omit<StackProps, 'direction'>) => JSX.Element` |
| VStack | 纵向堆叠（`direction` 固定为 `'column'`）  | `(props: Omit<StackProps, 'direction'>) => JSX.Element` |
| Spacer | 撑开剩余空间的弹性占位，把兄弟元素推向两端 | `(props: { className?: string }) => JSX.Element`        |

## 设计细节

- **令牌驱动**：`gap` / `align` / `justify` 全部映射到 Tailwind 令牌阶梯，不接受任意数值，杜绝「每个页面间距都不一样」
- **一维用 flex，二维用 grid**：这是组件库的布局约束，Stack 只做一维
- **语义化简写**：`HStack` / `VStack` 让方向在调用处即可读，`Spacer` 补齐「推到两端」的最后一环
- **可换标签**：`as` 允许渲染成 `ul` / `section` 等语义标签，同时保留全部布局行为
