import { DataTable, createDataTableColumnHelper } from '@agent-packages/shared-ui';

type Employee = { id: string; name: string; dept: string; score: number };

const helper = createDataTableColumnHelper<Employee>();

const columns = [
  helper.accessor('name', { header: '姓名' }),
  helper.accessor('dept', { header: '部门' }),
  helper.accessor('score', { header: '绩效分' }),
];

const depts = ['平台组', '设计组', '增长组', '数据组'];
const surnames = ['林', '周', '王', '陈', '刘', '赵', '孙', '钱', '李', '吴', '郑', '冯'];

const data: Employee[] = surnames.map((s, i) => ({
  id: `e${i + 1}`,
  name: `${s}同学`,
  dept: depts[i % depts.length],
  score: 62 + ((i * 17) % 38),
}));

export default function DataTablePagination() {
  return <DataTable columns={columns} data={data} pageSize={5} className="max-w-xl" />;
}
