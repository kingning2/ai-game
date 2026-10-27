import { useState } from 'react';
import { Button, Message, MessageList } from '@agent-packages/shared-ui';

/** 滚动跟随：贴底时新内容自动跟随；上滑后出现"回到底部"按钮，点击平滑滚回 */
export default function MessageScroll() {
  const [count, setCount] = useState(6);

  return (
    <div className="flex h-[380px] flex-col overflow-hidden rounded-xl bg-secondary-grouped-background shadow-card">
      <MessageList className="min-h-0 flex-1 px-4 py-4">
        {Array.from({ length: count }, (_, i) =>
          i % 2 ? (
            <Message key={i} role="assistant">
              第 {Math.ceil((i + 1) / 2)} 轮回复：这是用来撑出滚动距离的占位内容。
            </Message>
          ) : (
            <Message key={i} role="user">
              第 {i / 2 + 1} 条消息
            </Message>
          ),
        )}
      </MessageList>
      <div className="border-t border-separator/60 p-3">
        <Button variant="tinted" size="sm" onClick={() => setCount((c) => c + 2)}>
          追加两条消息（贴底时自动跟随）
        </Button>
      </div>
    </div>
  );
}
