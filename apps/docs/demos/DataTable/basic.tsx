import { DataTable, createDataTableColumnHelper } from '@agent-packages/shared-ui';

type Employee = { id: string; name: string; dept: string; city: string; age: number };

const helper = createDataTableColumnHelper<Employee>();

const columns = [
  helper.accessor('name', { header: '姓名' }),
  helper.accessor('dept', { header: '部门' }),
  helper.accessor('city', { header: '城市' }),
  helper.accessor('age', { header: '年龄' }),
];

const data: Employee[] = [
  { id: 'e1', name: '林小临', dept: '平台组', city: '杭州', age: 29 },
  { id: 'e2', name: '周小雨', dept: '设计组', city: '上海', age: 26 },
  { id: 'e3', name: '王一舟', dept: '数据组', city: '北京', age: 31 },
  { id: 'e4', name: '陈北望', dept: '增长组', city: '深圳', age: 27 },
  { id: 'e5', name: '刘星野', dept: '平台组', city: '杭州', age: 33 },
];

export default function DataTableBasic() {
  return <DataTable columns={columns} data={data} className="max-w-xl" />;
}
