import { useState } from 'react';
import { ThinkingBlock } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const CONTENT =
  '受控模式下展开状态完全由外部 open 决定，组件内的自动展开 / 收起不再生效。\n\n适合与外层布局联动：例如父容器折叠时强制收起思考面板。';

/** 受控模式：open 由外部状态驱动，onToggle 只做通知 */
export default function ThinkingControlled() {
  const [open, setOpen] = useState(false);

  return (
    <Stack gap={3}>
      <Stack direction="row" gap={2}>
        <button
          type="button"
          onClick={() => setOpen(true)}
          className="rounded-lg bg-tint px-3 py-1.5 text-xs font-medium text-white transition-opacity hover:opacity-90"
        >
          展开
        </button>
        <button
          type="button"
          onClick={() => setOpen(false)}
          className="rounded-lg bg-gray px-3 py-1.5 text-xs font-medium text-label transition-opacity hover:opacity-90"
        >
          收起
        </button>
        <span className="self-center font-mono text-xs text-tertiary-label">
          open = {String(open)}
        </span>
      </Stack>
      <ThinkingBlock
        content={CONTENT}
        thinking={false}
        duration={8}
        open={open}
        onToggle={setOpen}
      />
    </Stack>
  );
}
