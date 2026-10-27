import { useState } from 'react';
import { Lightbox } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const svgUri = (color: string, text: string) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="800"><rect width="100%" height="100%" fill="${color}"/><text x="50%" y="50%" fill="#ffffff" font-family="sans-serif" font-size="64" text-anchor="middle" dominant-baseline="middle">${text}</text></svg>`,
  )}`;

/** 受控模式：父组件持有 open 状态，Lightbox 只负责展示与回调 onClose */
export default function LightboxControlled() {
  const [open, setOpen] = useState(false);

  return (
    <Stack gap={3}>
      <Stack direction="row" gap={3} align="center">
        <button
          type="button"
          onClick={() => setOpen(true)}
          className="rounded-lg bg-tint px-3 py-1.5 text-xs font-medium text-white transition-opacity hover:opacity-90"
        >
          打开
        </button>
        <button
          type="button"
          onClick={() => setOpen(false)}
          className="rounded-lg bg-gray px-3 py-1.5 text-xs font-medium text-label transition-opacity hover:opacity-90"
        >
          关闭
        </button>
        <span className="font-mono text-xs text-tertiary-label">open = {String(open)}</span>
      </Stack>
      {open && (
        <Lightbox
          src={svgUri('%23d9544a', 'Controlled by Parent')}
          onClose={() => setOpen(false)}
        />
      )}
    </Stack>
  );
}
