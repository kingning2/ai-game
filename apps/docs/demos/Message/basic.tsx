import { Message, MessageList } from '@agent-packages/shared-ui';

/** 基础用法：混合式布局 —— user 气泡右对齐、assistant 通栏 + 操作条 + loading 态 */
export default function MessageBasic() {
  return (
    <div className="flex h-[380px] flex-col overflow-hidden rounded-xl bg-secondary-grouped-background shadow-card">
      <MessageList className="min-h-0 flex-1 px-4 py-4">
        <Message role="user" timestamp="14:02">
          帮我看下本月 ARK 账单花了多少
        </Message>
        <Message role="assistant" timestamp="14:02" onCopy={() => {}} onRegenerate={() => {}}>
          本月账单共 ¥12.40：推理 ¥9.80（doubao-seed-1.6）、Embedding ¥2.60。需要看按 Endpoint
          的拆分明细吗？
        </Message>
        <Message role="user" timestamp="14:03">
          要，按 Endpoint 拆一下
        </Message>
        <Message role="assistant" loading />
      </MessageList>
    </div>
  );
}
