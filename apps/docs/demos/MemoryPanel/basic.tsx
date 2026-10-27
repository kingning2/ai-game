import { MemoryPanel } from '@agent-packages/shared-ui';

/** 类型分档（fact/note/entity/plan）+ 置顶 + 框内检索（非受控，组件内过滤） */
export default function MemoryPanelBasic() {
  return (
    <MemoryPanel
      entries={[
        {
          id: 'm1',
          kind: 'fact',
          content: '续约客户 Acme Corp 的 SLA 违约次数为 2',
          time: '昨天 14:20',
          pinned: true,
        },
        {
          id: 'm2',
          kind: 'plan',
          content: '下季度前完成账单 webhook 重试策略改造',
          time: '周一 09:00',
        },
        { id: 'm3', kind: 'entity', content: 'Acme Corp · 续约联系人：王敏', time: '上周五' },
        { id: 'm4', kind: 'note', content: '用户偏好正式语气的邮件', time: '上周四' },
        {
          id: 'm5',
          kind: 'fact',
          content: 'Billing agent 的 webhook 超时阈值是 30s',
          time: '10月12日',
        },
      ]}
      onOpenEntry={(e) => window.alert(`打开记忆：${e.content}`)}
    />
  );
}
