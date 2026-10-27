import { useState } from 'react';
import { ApprovalGate } from '@agent-packages/shared-ui';

/** 结构化决策门：pending 显示选项组，点选后收拢为结果行（与 ApprovalCard 的 允许/拒绝 分工） */
export default function ApprovalGateBasic() {
  const [status, setStatus] = useState<'pending' | 'decided'>('pending');
  const [choice, setChoice] = useState<string>();

  return (
    <ApprovalGate
      title="Researcher 请求决策：纪要如何落地"
      description="搜索完成，找到两种可行路径，需要你拍板后继续"
      options={[
        { id: 'a', label: '拆分为三个子任务并行', description: '更快，但 token 消耗约 ×1.8' },
        { id: 'b', label: '单 agent 顺序完成', description: '更省，预计 4 分钟' },
      ]}
      status={status}
      decidedOptionId={choice}
      note={status === 'decided' ? '已通知下游 agent 按此执行' : undefined}
      onDecide={(id) => {
        setChoice(id);
        setStatus('decided');
      }}
    />
  );
}
