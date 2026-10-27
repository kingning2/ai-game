import { useState } from 'react';
import { Button, TaskPlan } from '@agent-packages/shared-ui';
import type { TaskStep } from '@agent-packages/shared-ui';

/** 推进交互：running → done → 下一步 running；全完成可重置。状态完全由调用方驱动 */
export default function TaskPlanAdvance() {
  const [steps, setSteps] = useState<TaskStep[]>([
    { title: '解析账单周期', status: 'done' },
    { title: '调用 ARK 拆分账单接口', status: 'running' },
    { title: '按 Endpoint 汇总金额', status: 'pending' },
    { title: '输出月度报告', status: 'pending' },
  ]);

  const advance = () => {
    setSteps((list) => {
      const ri = list.findIndex((s) => s.status === 'running');
      if (ri === -1)
        return list.map((s, i) => ({
          ...s,
          status: i === 0 ? ('running' as const) : ('pending' as const),
        }));
      const next = [...list];
      next[ri] = { ...next[ri], status: 'done' };
      if (next[ri + 1]) next[ri + 1] = { ...next[ri + 1], status: 'running' };
      return next;
    });
  };

  return (
    <div className="flex flex-col gap-2">
      <TaskPlan steps={steps} />
      <Button variant="tinted" size="sm" onClick={advance}>
        {steps.some((s) => s.status === 'running') ? '推进下一步' : '重新演示'}
      </Button>
    </div>
  );
}
