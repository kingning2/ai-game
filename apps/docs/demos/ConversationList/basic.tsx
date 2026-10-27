import { useState } from 'react';
import { ConversationList } from '@agent-packages/shared-ui';

/** 分组 + 高亮 + 行内重命名 + 删除：数据完全由调用方持有 */
// mock 数据在模块作用域生成：渲染期保持纯函数（react/purity）
const MOCK_CONVERSATIONS = [
  {
    id: 'c1',
    title: '本月账单拆解',
    preview: 'ep-202406…（Seedream）¥8.20…',
    time: '14:02',
    date: new Date(),
  },
  {
    id: 'c2',
    title: 'Seedream 出图排查',
    preview: '图像模糊的三个可能原因…',
    time: '11:47',
    date: new Date(),
  },
  {
    id: 'c3',
    title: '精调任务调参',
    preview: 'learning rate 降到 2e-5 后…',
    time: '昨天',
    date: new Date(Date.now() - 86_400_000),
  },
  {
    id: 'c4',
    title: 'Endpoint 迁移方案',
    preview: '旧 EP 下线前的灰度策略…',
    time: '周二',
    date: new Date(Date.now() - 3 * 86_400_000),
  },
  {
    id: 'c5',
    title: 'ArkCLI 升级记录',
    preview: '0.1.16 起 SSO 登录走 Gate 1+2…',
    time: '6/02',
    date: new Date(Date.now() - 12 * 86_400_000),
  },
];

export default function ConversationListBasic() {
  const [conversations, setConversations] = useState(MOCK_CONVERSATIONS);
  const [activeId, setActiveId] = useState('c1');

  return (
    <div className="max-w-sm rounded-xl bg-secondary-grouped-background p-2 shadow-card">
      <ConversationList
        conversations={conversations}
        activeId={activeId}
        onSelect={setActiveId}
        onRename={(id, title) =>
          setConversations((l) => l.map((c) => (c.id === id ? { ...c, title } : c)))
        }
        onDelete={(id) => setConversations((l) => l.filter((c) => c.id !== id))}
        onNew={() => {
          const id = `c${Date.now()}`;
          setConversations((l) => [{ id, title: '新会话', date: new Date() }, ...l]);
          setActiveId(id);
        }}
      />
    </div>
  );
}
