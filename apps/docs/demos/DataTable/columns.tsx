import { Badge, Button, DataTable, createDataTableColumnHelper } from '@agent-packages/shared-ui';

type Employee = {
  id: string;
  name: string;
  dept: string;
  status: 'active' | 'pending' | 'leave';
};

const helper = createDataTableColumnHelper<Employee>();

const statusTone = { active: 'green', pending: 'orange', leave: 'gray' } as const;
const statusText = { active: '在职', pending: '待入职', leave: '休假' };

const columns = [
  helper.accessor('name', { header: '姓名' }),
  helper.accessor('dept', { header: '部门' }),
  helper.accessor('status', {
    header: '状态',
    cell: (info) => <Badge tone={statusTone[info.getValue()]}>{statusText[info.getValue()]}</Badge>,
  }),
  helper.display({
    id: 'actions',
    header: '操作',
    cell: ({ row }) => (
      <Button variant="plain" size="sm" onClick={() => console.log('编辑', row.original.name)}>
        编辑
      </Button>
    ),
  }),
];

const data: Employee[] = [
  { id: 'e1', name: '林小临', dept: '平台组', status: 'active' },
  { id: 'e2', name: '周小雨', dept: '设计组', status: 'pending' },
  { id: 'e3', name: '王一舟', dept: '数据组', status: 'leave' },
  { id: 'e4', name: '陈北望', dept: '增长组', status: 'active' },
];

export default function DataTableColumns() {
  return <DataTable columns={columns} data={data} className="max-w-xl" />;
}
