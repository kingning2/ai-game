import { useState, type ReactNode } from 'react';
import {
  createColumnHelper,
  createCoreRowModel,
  createPaginatedRowModel,
  createSortedRowModel,
  flexRender,
  rowPaginationFeature,
  rowSortingFeature,
  tableFeatures,
  useTable,
  type ColumnDef,
  type RowData,
  type SortingState,
} from '@tanstack/react-table';
import { cn } from '../cn';

/**
 * 数据表格 —— TanStack Table v9（headless）+ Apple HIG 主题：
 * - 能力（排序/分页）由 TanStack features 提供，样式全部本地可控
 * - 点击表头排序（升 → 降 → 取消），tint 色指示箭头
 * - 传 pageSize 启用内置分页（轻分页栏：共 N 条 · x / y 页 + 上/下一页）
 * - 行 hover 高亮、onRowClick 可选中（selectedId 命中行加 tint 底色）
 * 列定义：
 *   const helper = createDataTableColumnHelper<Employee>();
 *   const columns = [helper.accessor('name', { header: '姓名' })];
 *   <DataTable columns={columns} data={rows} pageSize={5} onRowClick={...} />
 */

/** DataTable 内置 feature 集：核心 + 排序 + 分页。模块级构造（官方推荐静态化） */
export const dataTableFeatures = tableFeatures({
  coreRowModel: createCoreRowModel(),
  rowSortingFeature,
  rowPaginationFeature,
  sortedRowModel: createSortedRowModel(),
  paginatedRowModel: createPaginatedRowModel(),
});

/** features 感知的列定义 helper，避免手写 createColumnHelper<typeof dataTableFeatures, TData>() */
export function createDataTableColumnHelper<TData extends RowData>() {
  return createColumnHelper<typeof dataTableFeatures, TData>();
}

/** 列定义类型（供调用方显式标注列数组） */
export type DataTableColumnDef<TData extends RowData> = ColumnDef<
  typeof dataTableFeatures,
  TData,
  unknown
>;

export type DataTableProps<TData extends RowData> = {
  columns: DataTableColumnDef<TData>[];
  data: TData[];
  /** 传入即启用分页（每页行数） */
  pageSize?: number;
  /** 初始排序，如 [{ id: 'age', desc: true }] */
  initialSorting?: SortingState;
  onRowClick?: (row: TData) => void;
  /** 行选中标识（配合 getRowId），命中行加 tint 底色 */
  selectedId?: string;
  getRowId?: (row: TData, index: number) => string;
  empty?: ReactNode;
  className?: string;
};

function SortIndicator({ dir }: { dir: false | 'asc' | 'desc' }) {
  if (!dir) return <span className="inline-block w-3 text-[10px] text-transparent">▲</span>;
  return (
    <span className="inline-block w-3 text-[10px] text-tint">{dir === 'asc' ? '▲' : '▼'}</span>
  );
}

export function DataTable<TData extends RowData>({
  columns,
  data,
  pageSize,
  initialSorting,
  onRowClick,
  selectedId,
  getRowId,
  empty = '暂无数据',
  className,
}: DataTableProps<TData>) {
  const [sorting, setSorting] = useState<SortingState>(initialSorting ?? []);
  const [pagination, setPagination] = useState({ pageIndex: 0, pageSize: pageSize ?? 10 });

  const table = useTable({
    features: dataTableFeatures,
    columns,
    data,
    state: { sorting, pagination },
    onSortingChange: setSorting,
    onPaginationChange: setPagination,
    getRowId,
  });

  const rows = table.getRowModel().rows;
  const pageCount = table.getPageCount();
  const showPager = pageSize !== undefined && pageCount > 1;

  return (
    <div
      className={cn('overflow-hidden rounded-xl border border-separator bg-background', className)}
    >
      <table className="w-full border-collapse text-[13px]">
        <thead>
          {table.getHeaderGroups().map((hg) => (
            <tr key={hg.id}>
              {hg.headers.map((header) => {
                const canSort = header.column.getCanSort();
                const dir = header.column.getIsSorted();
                return (
                  <th
                    key={header.id}
                    className="border-b border-separator bg-tertiary-fill px-3 py-2 text-start font-semibold text-secondary-label"
                  >
                    {canSort ? (
                      <button
                        type="button"
                        onClick={header.column.getToggleSortingHandler()}
                        className="flex items-center gap-1 transition-opacity hover:opacity-70"
                      >
                        {flexRender(header.column.columnDef.header, header.getContext())}
                        <SortIndicator dir={dir} />
                      </button>
                    ) : (
                      flexRender(header.column.columnDef.header, header.getContext())
                    )}
                  </th>
                );
              })}
            </tr>
          ))}
        </thead>
        <tbody>
          {rows.length === 0 ? (
            <tr>
              <td colSpan={columns.length} className="px-3 py-10 text-center text-tertiary-label">
                {empty}
              </td>
            </tr>
          ) : (
            rows.map((row) => (
              <tr
                key={row.id}
                onClick={() => onRowClick?.(row.original)}
                className={cn(
                  'border-b border-separator transition-colors last:border-b-0',
                  'hover:bg-tertiary-fill/60',
                  onRowClick && 'cursor-pointer',
                  selectedId !== undefined && row.id === selectedId && 'bg-tint/5',
                )}
              >
                {row.getAllCells().map((cell) => (
                  <td key={cell.id} className="px-3 py-2.5 align-top text-label">
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>

      {showPager && (
        <div className="flex items-center justify-between border-t border-separator px-3 py-2 text-xs text-secondary-label">
          <span>
            共 {data.length} 条 · 第 {pagination.pageIndex + 1} / {pageCount} 页
          </span>
          <div className="flex gap-1">
            <button
              type="button"
              disabled={!table.getCanPreviousPage()}
              onClick={() => table.previousPage()}
              className="rounded-lg px-2 py-1 transition-colors hover:bg-tertiary-fill disabled:opacity-40"
            >
              ‹ 上一页
            </button>
            <button
              type="button"
              disabled={!table.getCanNextPage()}
              onClick={() => table.nextPage()}
              className="rounded-lg px-2 py-1 transition-colors hover:bg-tertiary-fill disabled:opacity-40"
            >
              下一页 ›
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
