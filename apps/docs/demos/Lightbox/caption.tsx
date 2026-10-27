import { useState } from 'react';
import { Lightbox } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const svgUri = (color: string, text: string) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="800"><rect width="100%" height="100%" fill="${color}"/><text x="50%" y="50%" fill="#ffffff" font-family="sans-serif" font-size="64" text-anchor="middle" dominant-baseline="middle">${text}</text></svg>`,
  )}`;

/** caption 显示在图片下方，常用于文件名 */
export default function LightboxCaption() {
  const [open, setOpen] = useState(false);

  return (
    <Stack direction="row" gap={3}>
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="rounded-lg bg-tint px-3 py-1.5 text-xs font-medium text-white transition-opacity hover:opacity-90"
      >
        预览 周报-图表.png
      </button>
      {open && (
        <Lightbox
          src={svgUri('%2334a853', 'Weekly Report Chart')}
          alt="周报图表"
          caption="周报-图表.png（2.4 MB）"
          onClose={() => setOpen(false)}
        />
      )}
    </Stack>
  );
}
