import { useEffect, useState } from 'react';
import { Markdown } from '@agent-packages/shared-ui';

const CHUNKS = [
  '### 流式渲染\n\nAI 回答通过 SSE 分片到达，',
  'fence 未闭合时 hook 会**自动补闭合**，',
  '避免后半文被吞进代码块。\n\n```ts\nconst ok = true;\n```',
];

export default function MarkdownStreaming() {
  const [text, setText] = useState('');
  const [done, setDone] = useState(false);

  useEffect(() => {
    let i = 0;
    const timer = setInterval(() => {
      if (i >= CHUNKS.length) {
        clearInterval(timer);
        setDone(true);
        return;
      }
      setText((prev) => prev + CHUNKS[i++]);
    }, 900);
    return () => clearInterval(timer);
  }, []);

  return (
    <div className="flex flex-col gap-2">
      <div className="rounded-xl border border-separator bg-background p-4">
        <Markdown text={text} />
      </div>
      <span className="text-xs text-tertiary-label">
        {done ? '流式结束（累计 3 个 chunk）' : '接收中…'}
      </span>
    </div>
  );
}
