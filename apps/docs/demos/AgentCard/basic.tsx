import { useState } from 'react';
import { AgentCard, type AgentCardData } from '@agent-packages/shared-ui';

/** 五种状态 + 失败卡的重跑交互（scrimui：失败是一张卡不是一次宕机） */
const AGENTS: AgentCardData[] = [
  {
    id: 'a1',
    name: 'Orchestrator',
    role: 'orchestrator',
    model: 'doubao-seed-1.6',
    status: 'running',
    activity: '正在协调三个子任务',
    tokens: 48200,
    costUsd: 0.31,
    durationMs: 134_000,
  },
  {
    id: 'a2',
    name: 'Researcher',
    role: 'research',
    status: 'done',
    tokens: 96400,
    costUsd: 0.42,
    durationMs: 96_000,
  },
  { id: 'a3', name: 'Writer', role: 'writing', status: 'waiting' },
  { id: 'a4', name: 'Reviewer', role: 'review', status: 'idle' },
  {
    id: 'a5',
    name: 'Billing',
    role: 'billing',
    status: 'failed',
    activity: 'webhook 超时',
    tokens: 8900,
    costUsd: 0.04,
    durationMs: 22_000,
  },
];

export default function AgentCardBasic() {
  const [statuses, setStatuses] = useState(
    Object.fromEntries(AGENTS.map((a) => [a.id, a.status])) as Record<
      string,
      AgentCardData['status']
    >,
  );

  return (
    <div className="flex flex-wrap gap-2">
      {AGENTS.map((agent) => (
        <AgentCard
          key={agent.id}
          agent={{ ...agent, status: statuses[agent.id] }}
          className="min-w-[16rem] flex-1 basis-[16rem]"
          onRerun={
            agent.status === 'failed'
              ? () => {
                  setStatuses((s) => ({ ...s, [agent.id]: 'running' }));
                  setTimeout(() => setStatuses((s) => ({ ...s, [agent.id]: 'done' })), 1800);
                }
              : undefined
          }
        />
      ))}
    </div>
  );
}
