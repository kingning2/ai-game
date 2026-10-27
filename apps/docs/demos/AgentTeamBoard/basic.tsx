import { useState } from 'react';
import { AgentTeamBoard, type AgentCardData } from '@agent-packages/shared-ui';

const AGENTS: AgentCardData[] = [
  {
    id: 'orchestrator',
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
    id: 'researcher',
    name: 'Researcher',
    role: 'research',
    status: 'done',
    tokens: 96400,
    costUsd: 0.42,
    durationMs: 96_000,
  },
  {
    id: 'writer',
    name: 'Writer',
    role: 'writing',
    status: 'running',
    activity: '正在起草纪要',
    tokens: 31800,
    costUsd: 0.14,
    durationMs: 41_000,
  },
  {
    id: 'billing',
    name: 'Billing',
    role: 'billing',
    status: 'failed',
    activity: 'webhook 超时',
    tokens: 8900,
    costUsd: 0.04,
    durationMs: 22_000,
  },
];

/** fleet 头部汇总（运行数/待审批/总成本）+ 选中卡片切换详情（永不换页） */
export default function AgentTeamBoardBasic() {
  const [selectedId, setSelectedId] = useState('orchestrator');

  return (
    <AgentTeamBoard
      agents={AGENTS}
      selectedId={selectedId}
      onSelectAgent={setSelectedId}
      onRerunAgent={(id) => window.alert(`重跑 ${id}（真实场景接入调用方状态机）`)}
      pendingApprovals={1}
      onOpenApprovals={() => window.alert('定位到审批门（决策内联在 agent 时间线里）')}
    />
  );
}
