import { DataTable, VStack, createDataTableColumnHelper } from '@agent-packages/shared-ui';

type Employee = { id: string; name: string; dept: string };

const helper = createDataTableColumnHelper<Employee>();

const columns = [
  helper.accessor('name', { header: '姓名' }),
  helper.accessor('dept', { header: '部门' }),
];

export default function DataTableEmpty() {
  return (
    <VStack gap={4} className="max-w-md">
      <DataTable columns={columns} data={[]} />
      <DataTable
        columns={columns}
        data={[]}
        empty={
          <VStack align="center" gap={1}>
            <span className="text-sm text-label">没有匹配的成员</span>
            <span className="text-xs text-tertiary-label">试试调整筛选条件或清空搜索词</span>
          </VStack>
        }
      />
    </VStack>
  );
}
