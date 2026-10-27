import { AgentTimeline } from '@agent-packages/shared-ui';

/** 事件只追加不覆盖：Billing 超时（e4）被重试事件（e5）接替，e4 保留划线历史供审计 */
export default function AgentTimelineBasic() {
  return (
    <AgentTimeline
      title="运行时间线 · Orchestrator"
      events={[
        {
          id: 'e1',
          kind: 'spawn',
          title: '拉起 3 个子 agent',
          detail: 'researcher / writer / billing',
          time: '09:39:02',
        },
        {
          id: 'e2',
          kind: 'tool',
          title: 'Researcher · web_search',
          detail: '2 条查询 · 8 个来源 · 41s',
          time: '09:39:40',
        },
        {
          id: 'e3',
          kind: 'handoff',
          title: 'Researcher → Writer 交接',
          detail: '携带 3 项上下文，留下 2 项',
          time: '09:40:41',
        },
        {
          id: 'e4',
          kind: 'error',
          title: 'Billing · webhook 超时',
          detail: 'POST /hooks/acme 超过 30s 未响应',
          time: '09:41:07',
        },
        {
          id: 'e5',
          kind: 'retry',
          title: 'Billing 重跑（第 1 次）',
          detail: '超时阈值提高到 60s',
          linkedId: 'e4',
          time: '09:41:15',
        },
        {
          id: 'e6',
          kind: 'approval',
          title: '等待决策：纪要如何落地',
          detail: '2 个候选方案',
          time: '09:41:30',
        },
      ]}
    />
  );
}
