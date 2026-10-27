import { useState } from 'react';
import { Lightbox } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

/** 内联 SVG 占位图（data URI），demo 禁用外链图片 */
const svgUri = (color: string, text: string) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="800"><rect width="100%" height="100%" fill="${color}"/><text x="50%" y="50%" fill="#ffffff" font-family="sans-serif" font-size="64" text-anchor="middle" dominant-baseline="middle">${text}</text></svg>`,
  )}`;

export default function LightboxBasic() {
  const [open, setOpen] = useState(false);

  return (
    <Stack direction="row" gap={3}>
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="rounded-lg bg-tint px-3 py-1.5 text-xs font-medium text-white transition-opacity hover:opacity-90"
      >
        打开图片预览
      </button>
      {open && (
        <Lightbox src={svgUri('%234a90d9', 'Screenshot 1200×800')} onClose={() => setOpen(false)} />
      )}
    </Stack>
  );
}
