import { useState } from 'react';
import { Typewriter } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const TEXT = '光标行为可配置：默认打完隐藏；hideCursorOnDone=false 时光标常驻结尾，适合多轮对话。';

export default function TypewriterCursor() {
  // 换 key 重挂载以重播
  const [run, setRun] = useState(0);

  return (
    <Stack gap={4}>
      <Stack gap={1}>
        <span className="text-xs text-tertiary-label">默认（打完后光标消失）</span>
        <Typewriter key={`a-${run}`} text={TEXT} />
      </Stack>
      <Stack gap={1}>
        <span className="text-xs text-tertiary-label">hideCursorOnDone={false}（光标常驻）</span>
        <Typewriter key={`b-${run}`} text={TEXT} hideCursorOnDone={false} />
      </Stack>
      <Stack gap={1}>
        <span className="text-xs text-tertiary-label">cursor={false}（无光标）</span>
        <Typewriter key={`c-${run}`} text={TEXT} cursor={false} />
      </Stack>
      <button
        type="button"
        onClick={() => setRun((r) => r + 1)}
        className="w-fit rounded-lg bg-tint px-3 py-1.5 text-xs font-medium text-white transition-opacity hover:opacity-90"
      >
        重播
      </button>
    </Stack>
  );
}
