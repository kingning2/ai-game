import { useState } from 'react';
import { Button, ApprovalCard } from '@agent-packages/shared-ui';

/** 状态流转：pending 按钮组 → approved/denied 结果行，note 随操作不同 */
export default function ApprovalCardBasic() {
  const [approval, setApproval] = useState<{
    status: 'pending' | 'approved' | 'denied';
    note?: string;
  }>({
    status: 'pending',
  });

  return (
    <div className="flex flex-col gap-2">
      <ApprovalCard
        request={{
          title: '执行 bash 命令：rm -rf dist/',
          description: '删除构建产物目录，该操作不可撤销。',
          kind: 'bash',
        }}
        status={approval.status}
        note={approval.note}
        onApprove={() => setApproval({ status: 'approved', note: '仅本次允许' })}
        onAlwaysAllow={() => setApproval({ status: 'approved', note: '本次会话内不再询问' })}
        onDeny={() => setApproval({ status: 'denied' })}
      />
      {approval.status !== 'pending' && (
        <Button variant="plain" size="sm" onClick={() => setApproval({ status: 'pending' })}>
          重置
        </Button>
      )}
    </div>
  );
}
