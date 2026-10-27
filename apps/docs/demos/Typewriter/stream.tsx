import { useEffect, useRef, useState } from 'react';
import type { TypewriterHandle } from '@agent-packages/shared-ui';
import { Typewriter } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const CHUNKS = [
  '正在检索知识库…\n',
  '找到 3 篇相关文档。\n',
  '综合来看：',
  '流式接口每收到一段 token，',
  '直接调用 ref.push() 喂入即可，',
  '无需经过 state 中转。',
];

/** 非受控用法：模拟 SSE，onmessage 到达一个 chunk 就 push 一段 */
export default function TypewriterStream() {
  const ref = useRef<TypewriterHandle>(null);
  const [started, setStarted] = useState(false);

  useEffect(() => {
    if (!started) return;
    let i = 0;
    ref.current?.reset();
    const timer = setInterval(() => {
      if (i >= CHUNKS.length) {
        clearInterval(timer);
        return;
      }
      ref.current?.push(CHUNKS[i]);
      i += 1;
    }, 300);
    return () => clearInterval(timer);
  }, [started]);

  return (
    <Stack gap={3}>
      <Typewriter ref={ref} speed={80} />
      <Stack direction="row" gap={2}>
        <button
          type="button"
          onClick={() => setStarted(true)}
          className="rounded-lg bg-tint px-3 py-1.5 text-xs font-medium text-white transition-opacity hover:opacity-90"
        >
          开始推送
        </button>
        <button
          type="button"
          onClick={() => ref.current?.skip()}
          className="rounded-lg bg-gray px-3 py-1.5 text-xs font-medium text-label transition-opacity hover:opacity-90"
        >
          跳过动画
        </button>
      </Stack>
    </Stack>
  );
}
