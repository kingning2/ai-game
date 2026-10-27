import { type AttachmentItem } from '@agent-packages/shared-ui';
import { AttachmentList } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const svgUri = (color: string, text: string) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="800" height="800"><rect width="100%" height="100%" fill="${color}"/><text x="50%" y="50%" fill="#ffffff" font-family="sans-serif" font-size="48" text-anchor="middle" dominant-baseline="middle">${text}</text></svg>`,
  )}`;

const ITEMS: AttachmentItem[] = [
  { id: 's1', url: svgUri('%234a90d9', 'IMG'), name: 'a.png' },
  { id: 's2', url: svgUri('%2334a853', 'IMG'), name: 'b.png' },
];

/** size 控制缩略图边长（px），默认 64 */
export default function AttachmentSize() {
  return (
    <Stack gap={5}>
      <Stack gap={1}>
        <span className="text-xs text-tertiary-label">size=40（输入框内联）</span>
        <AttachmentList items={ITEMS} size={40} />
      </Stack>
      <Stack gap={1}>
        <span className="text-xs text-tertiary-label">size=64（默认）</span>
        <AttachmentList items={ITEMS} />
      </Stack>
      <Stack gap={1}>
        <span className="text-xs text-tertiary-label">size=96（大图模式）</span>
        <AttachmentList items={ITEMS} size={96} />
      </Stack>
    </Stack>
  );
}
