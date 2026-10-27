import { useState } from 'react';
import type { AttachmentItem } from '@agent-packages/shared-ui';
import { AttachmentList } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const svgUri = (color: string, text: string) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="800" height="800"><rect width="100%" height="100%" fill="${color}"/><text x="50%" y="50%" fill="#ffffff" font-family="sans-serif" font-size="48" text-anchor="middle" dominant-baseline="middle">${text}</text></svg>`,
  )}`;

/** 传入 onRemove 后 hover 显示 ×，点击按 id 回调移除 */
export default function AttachmentRemove() {
  const [items, setItems] = useState<AttachmentItem[]>([
    { id: 'r1', url: svgUri('%234a90d9', 'PNG'), name: '设计稿.png' },
    { id: 'r2', url: svgUri('%2334a853', 'JPG'), name: '会议白板.jpg' },
    { id: 'r3', url: svgUri('%23d9544a', 'PDF'), name: '合同扫描.pdf' },
  ]);

  return (
    <Stack gap={2}>
      <AttachmentList
        items={items}
        onRemove={(id) => setItems((list) => list.filter((a) => a.id !== id))}
      />
      <span className="text-xs text-tertiary-label">
        悬停缩略图显示移除按钮（剩余 {items.length} 张）
      </span>
    </Stack>
  );
}
