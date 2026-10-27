import { useState } from 'react';
import type { AttachmentItem } from '@agent-packages/shared-ui';
import { Lightbox } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const svgUri = (color: string, text: string) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="800"><rect width="100%" height="100%" fill="${color}"/><text x="50%" y="50%" fill="#ffffff" font-family="sans-serif" font-size="64" text-anchor="middle" dominant-baseline="middle">${text}</text></svg>`,
  )}`;

const IMAGES: AttachmentItem[] = [
  { id: '1', url: svgUri('%234a90d9', 'IMG_001'), name: 'IMG_001.png' },
  { id: '2', url: svgUri('%2334a853', 'IMG_002'), name: 'IMG_002.png' },
  { id: '3', url: svgUri('%23e6a23c', 'IMG_003'), name: 'IMG_003.png' },
];

/** 缩略图网格：点击任意一张全屏预览，同一时间只有一个 Lightbox 实例 */
export default function LightboxGallery() {
  const [current, setCurrent] = useState<AttachmentItem | null>(null);

  return (
    <Stack gap={3}>
      <Stack direction="row" gap={2} wrap>
        {IMAGES.map((img) => (
          <button
            key={img.id}
            type="button"
            aria-label={`预览 ${img.name}`}
            onClick={() => setCurrent(img)}
            className="size-16 cursor-zoom-in overflow-hidden rounded-lg border border-separator"
          >
            <img src={img.url} alt={img.name} className="size-full object-cover" />
          </button>
        ))}
      </Stack>
      {current && (
        <Lightbox
          src={current.url}
          alt={current.name}
          caption={current.name}
          onClose={() => setCurrent(null)}
        />
      )}
    </Stack>
  );
}
