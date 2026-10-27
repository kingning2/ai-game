import { DataTable, createDataTableColumnHelper } from '@agent-packages/shared-ui';

type Employee = { id: string; name: string; dept: string; score: number; age: number };

const helper = createDataTableColumnHelper<Employee>();

const columns = [
  helper.accessor('name', { header: '姓名' }),
  helper.accessor('dept', { header: '部门' }),
  helper.accessor('score', { header: '绩效分' }),
  helper.accessor('age', { header: '年龄' }),
];

const data: Employee[] = [
  { id: 'e1', name: '林小临', dept: '平台组', score: 92, age: 29 },
  { id: 'e2', name: '周小雨', dept: '设计组', score: 87, age: 26 },
  { id: 'e3', name: '王一舟', dept: '数据组', score: 95, age: 31 },
  { id: 'e4', name: '陈北望', dept: '增长组', score: 78, age: 27 },
  { id: 'e5', name: '刘星野', dept: '平台组', score: 84, age: 33 },
];

export default function DataTableSort() {
  return (
    <DataTable
      columns={columns}
      data={data}
      initialSorting={[{ id: 'score', desc: true }]}
      className="max-w-xl"
    />
  );
}
