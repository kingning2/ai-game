# DataTable 数据表格

基于 TanStack Table v9（headless）封装的数据表格，Apple HIG 主题：点击表头排序、轻量分页栏、行 hover 高亮与选中态，能力由 TanStack features 提供，样式全部本地可控。

## 何时使用

- 结构化数据的列表展示，需要列对齐、可扫描的表格形态
- 需要点击表头按列排序（升 → 降 → 取消）
- 数据量较大需要前端分页（传 `pageSize` 即启用）
- 需要点击行查看详情或标记选中行
- 需要自定义单元格渲染（状态徽标、操作按钮等）时，通过列定义的 `cell` 完成

## 代码演示

### 基础用法

用 `createDataTableColumnHelper<T>()` 声明列，`columns` + `data` 两个必需属性即可出表。

<DemoPreview name="DataTableBasic" />

<<< ../demos/DataTable/basic.tsx

### 自定义单元格与操作列

`accessor` 列的 `cell` 可渲染任意节点（这里是状态徽标）；`display` 列没有数据字段，适合放操作按钮，且天然不可排序。

<DemoPreview name="DataTableColumns" />

<<< ../demos/DataTable/columns.tsx

### 排序

可排列表头点击循环切换：升序 → 降序 → 取消，tint 色箭头指示方向；`initialSorting` 设置初始排序。

<DemoPreview name="DataTableSort" />

<<< ../demos/DataTable/sort.tsx

### 分页

传入 `pageSize` 启用内置分页（「共 N 条 · x / y 页」+ 上/下一页）；只有一页时分页栏自动隐藏。

<DemoPreview name="DataTablePagination" />

<<< ../demos/DataTable/pagination.tsx

### 行选中

`onRowClick` + `selectedId` + `getRowId` 三件套：命中行加 tint 底色，选中状态由外部 `useState` 管理。

<DemoPreview name="DataTableSelection" />

<<< ../demos/DataTable/selection.tsx

### 空态

数据为空时整行渲染 `empty` 内容，默认文案「暂无数据」，可传任意节点自定义。

<DemoPreview name="DataTableEmpty" />

<<< ../demos/DataTable/empty.tsx

### 组合使用

排序、分页、单元格定制与行选中可以自由叠加 —— headless 设计让各项能力互不干扰。

<DemoPreview name="DataTableFull" />

<<< ../demos/DataTable/full.tsx

## API

### DataTableProps&lt;TData&gt;

| 属性           | 说明                                                             | 类型                                    | 默认值       |
| -------------- | ---------------------------------------------------------------- | --------------------------------------- | ------------ |
| columns        | 列定义数组，推荐用 `createDataTableColumnHelper<TData>()` 构造   | `DataTableColumnDef<TData>[]`           | -            |
| data           | 行数据数组                                                       | `TData[]`                               | -            |
| pageSize       | 传入即启用内置分页（每页行数）                                   | `number`                                | -            |
| initialSorting | 初始排序，如 `[{ id: 'age', desc: true }]`                       | `SortingState`                          | `[]`         |
| onRowClick     | 行点击回调，参数为该行原始数据                                   | `(row: TData) => void`                  | -            |
| selectedId     | 行选中标识（与 `getRowId` 生成的行 id 比较），命中行加 tint 底色 | `string`                                | -            |
| getRowId       | 自定义行 id 生成函数                                             | `(row: TData, index: number) => string` | -            |
| empty          | 空数据提示节点，渲染在整行合并单元格中                           | `ReactNode`                             | `'暂无数据'` |
| className      | 附加类名，可覆盖圆角、最大宽度等                                 | `string`                                | -            |

### 其他导出

| 导出                        | 说明                                                                                           | 类型                                                  |
| --------------------------- | ---------------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| createDataTableColumnHelper | features 感知的列定义 helper，避免手写 `createColumnHelper<typeof dataTableFeatures, TData>()` | `<TData extends RowData>() => ColumnHelper`           |
| DataTableColumnDef          | 列定义类型，供调用方显式标注列数组                                                             | `ColumnDef<typeof dataTableFeatures, TData, unknown>` |
| dataTableFeatures           | 内置 feature 集（core + 排序 + 分页），模块级构造，官方推荐静态化                              | `TableFeatures`                                       |

## 设计细节

- **headless 分层**：排序 / 分页状态机由 TanStack Table v9 features 驱动，组件只负责渲染；`features` 模块级构造一次，所有表格实例共享
- **排序循环**：点击表头 升序 → 降序 → 取消；未排序列也渲染一个透明箭头占位，表头对齐不跳动
- **轻分页栏**：左「共 N 条 · x / y 页」、右「上/下一页」，单页时隐藏，不引入完整分页组件
- **行选中语义**：`selectedId` 与 `getRowId` 的行 id 精确匹配加 `bg-tint/5`，与 hover 的 `bg-tertiary-fill/60` 分层不冲突
- **空态兜底**：空数据渲染 `colSpan` 全宽单元格，不会出现布局塌陷
