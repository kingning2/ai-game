import { useState } from 'react';
import { Lightbox } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const svgUri = (color: string, text: string) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="800"><rect width="100%" height="100%" fill="${color}"/><text x="50%" y="50%" fill="#ffffff" font-family="sans-serif" font-size="64" text-anchor="middle" dominant-baseline="middle">${text}</text></svg>`,
  )}`;

/** className 覆盖遮罩：加深底色、去掉毛玻璃 */
export default function LightboxCustom() {
  const [open, setOpen] = useState(false);

  return (
    <Stack direction="row" gap={3}>
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="rounded-lg bg-tint px-3 py-1.5 text-xs font-medium text-white transition-opacity hover:opacity-90"
      >
        深色遮罩预览
      </button>
      {open && (
        <Lightbox
          src={svgUri('%237b68ee', 'Custom Backdrop')}
          className="bg-black/90 backdrop-blur-none"
          onClose={() => setOpen(false)}
        />
      )}
    </Stack>
  );
}
