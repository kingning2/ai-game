import { useState } from 'react';
import { DataTable, VStack, createDataTableColumnHelper } from '@agent-packages/shared-ui';

type Employee = { id: string; name: string; dept: string; city: string };

const helper = createDataTableColumnHelper<Employee>();

const columns = [
  helper.accessor('name', { header: '姓名' }),
  helper.accessor('dept', { header: '部门' }),
  helper.accessor('city', { header: '城市' }),
];

const data: Employee[] = [
  { id: 'e1', name: '林小临', dept: '平台组', city: '杭州' },
  { id: 'e2', name: '周小雨', dept: '设计组', city: '上海' },
  { id: 'e3', name: '王一舟', dept: '数据组', city: '北京' },
  { id: 'e4', name: '陈北望', dept: '增长组', city: '深圳' },
];

export default function DataTableSelection() {
  const [selected, setSelected] = useState<string | null>('e2');
  return (
    <VStack gap={2} className="max-w-xl">
      <DataTable
        columns={columns}
        data={data}
        getRowId={(row) => row.id}
        selectedId={selected ?? undefined}
        onRowClick={(row) => setSelected(row.id)}
      />
      <span className="text-xs text-tertiary-label">
        当前选中：{data.find((d) => d.id === selected)?.name ?? '（无）'}
      </span>
    </VStack>
  );
}
