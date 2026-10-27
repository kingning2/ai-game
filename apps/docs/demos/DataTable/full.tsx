import { useState } from 'react';
import { Badge, DataTable, VStack, createDataTableColumnHelper } from '@agent-packages/shared-ui';

type Employee = {
  id: string;
  name: string;
  dept: string;
  score: number;
  status: 'active' | 'pending' | 'leave';
};

const helper = createDataTableColumnHelper<Employee>();

const statusTone = { active: 'green', pending: 'orange', leave: 'gray' } as const;
const statusText = { active: '在职', pending: '待入职', leave: '休假' };

const columns = [
  helper.accessor('name', { header: '姓名' }),
  helper.accessor('dept', { header: '部门' }),
  helper.accessor('score', { header: '绩效分' }),
  helper.accessor('status', {
    header: '状态',
    cell: (info) => <Badge tone={statusTone[info.getValue()]}>{statusText[info.getValue()]}</Badge>,
  }),
];

const depts = ['平台组', '设计组', '增长组', '数据组'];
const surnames = ['林', '周', '王', '陈', '刘', '赵', '孙', '钱', '李', '吴', '郑', '冯'];
const statuses = ['active', 'pending', 'leave'] as const;

const data: Employee[] = surnames.map((s, i) => ({
  id: `e${i + 1}`,
  name: `${s}同学`,
  dept: depts[i % depts.length],
  score: 62 + ((i * 17) % 38),
  status: statuses[i % statuses.length],
}));

export default function DataTableFull() {
  const [selected, setSelected] = useState<string | null>(null);
  return (
    <VStack gap={2} className="max-w-2xl">
      <DataTable
        columns={columns}
        data={data}
        pageSize={6}
        initialSorting={[{ id: 'score', desc: true }]}
        getRowId={(row) => row.id}
        selectedId={selected ?? undefined}
        onRowClick={(row) => setSelected(row.id)}
      />
      <span className="text-xs text-tertiary-label">
        排序、分页、选中组合使用；点击行选中，点击表头切换排序。
      </span>
    </VStack>
  );
}
