import { TaskPlan } from '@agent-packages/shared-ui';

const STEPS = [
  { title: '解析账单周期', detail: 'billing period: 2025-06', status: 'done' as const },
  {
    title: '调用 ARK 拆分账单接口',
    detail: 'arkcli billing list --month 2025-06',
    status: 'running' as const,
  },
  { title: '按 Endpoint 汇总金额', status: 'pending' as const },
  { title: '生成趋势图表', detail: '上一步失败可标记 failed', status: 'failed' as const },
  { title: '输出月度报告', status: 'pending' as const },
];

/** 四态步骤：done / running / pending / failed，进度头自动统计 */
export default function TaskPlanBasic() {
  return <TaskPlan steps={STEPS} />;
}
