import { ApprovalCard } from '@agent-packages/shared-ui';

/** 结果态静态展示：approved / denied（无按钮组，右上角状态图标） */
export default function ApprovalCardSettled() {
  return (
    <div className="flex flex-col gap-2.5">
      <ApprovalCard
        request={{ title: '读取文件 packages/shared-ui/src/Message/index.tsx', kind: 'file' }}
        status="approved"
        note="本次会话内不再询问"
      />
      <ApprovalCard
        request={{
          title: '向 api.example.com 发送 POST 请求',
          description: '包含账单明细数据。',
          kind: 'network',
        }}
        status="denied"
      />
    </div>
  );
}
