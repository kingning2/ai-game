import { AgentTree } from '@agent-packages/shared-ui';

/** orchestrator → subagent 层级：缩进参考线表达委派与依赖 */
export default function AgentTreeBasic() {
  return (
    <AgentTree
      nodes={[
        {
          id: 'orchestrator',
          name: 'Orchestrator',
          role: 'orchestrator',
          status: 'running',
          activity: '协调三个子任务',
          children: [
            { id: 't-research', name: 'Researcher', role: 'research', status: 'done' },
            {
              id: 't-write',
              name: 'Writer',
              role: 'writing',
              status: 'running',
              activity: '正在起草纪要',
            },
            {
              id: 't-billing',
              name: 'Billing',
              role: 'billing',
              status: 'failed',
              activity: 'webhook 超时',
            },
          ],
        },
      ]}
    />
  );
}
