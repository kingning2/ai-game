import { useState } from 'react';
import type { AttachmentItem } from '@agent-packages/shared-ui';
import { AttachmentList } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const svgUri = (color: string, text: string) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="800" height="800"><rect width="100%" height="100%" fill="${color}"/><text x="50%" y="50%" fill="#ffffff" font-family="sans-serif" font-size="48" text-anchor="middle" dominant-baseline="middle">${text}</text></svg>`,
  )}`;

const INITIAL: AttachmentItem[] = [
  { id: 'a1', url: svgUri('%234a90d9', 'PNG'), name: 'screenshot.png' },
  { id: 'a2', url: svgUri('%2334a853', 'JPG'), name: 'photo.jpg' },
  { id: 'a3', url: svgUri('%23e6a23c', 'SVG'), name: 'diagram.svg' },
];

/** 基础用法：点击缩略图全屏预览（内置 Lightbox） */
export default function AttachmentBasic() {
  const [items] = useState<AttachmentItem[]>(INITIAL);

  return (
    <Stack gap={2}>
      <AttachmentList items={items} />
      <span className="text-xs text-tertiary-label">点击任意缩略图全屏预览</span>
    </Stack>
  );
}
