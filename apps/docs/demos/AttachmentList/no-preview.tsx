import { type AttachmentItem } from '@agent-packages/shared-ui';
import { AttachmentList } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const svgUri = (color: string, text: string) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="800" height="800"><rect width="100%" height="100%" fill="${color}"/><text x="50%" y="50%" fill="#ffffff" font-family="sans-serif" font-size="48" text-anchor="middle" dominant-baseline="middle">${text}</text></svg>`,
  )}`;

const ITEMS: AttachmentItem[] = [
  { id: 'n1', url: svgUri('%234a90d9', 'PNG'), name: '截图.png' },
  { id: 'n2', url: svgUri('%237b68ee', 'SVG'), name: '架构图.svg' },
];

/** preview=false 时纯展示，不可点击预览 */
export default function AttachmentNoPreview() {
  return (
    <Stack gap={2}>
      <AttachmentList items={ITEMS} preview={false} />
      <span className="text-xs text-tertiary-label">仅展示缩略图，点击无预览</span>
    </Stack>
  );
}
