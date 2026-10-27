import { useEffect, useRef, useState } from 'react';
import { ThinkingBlock } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const SEGMENTS = [
  '用户想了解进度条组件的用法，',
  '先拆解问题：需要覆盖基础展示、动态更新和无障碍。\n',
  '进度条是纯展示组件，',
  '关键点在于宽度过渡的流畅性和取值钳制。\n',
  '结论：直接给出 4 个场景示例即可。',
];

/** 完整生命周期：思考中流式追加 → 结束自动收起（autoCollapse） */
export default function ThinkingStreaming() {
  const [content, setContent] = useState('');
  const [thinking, setThinking] = useState(true);
  const idxRef = useRef(0);

  useEffect(() => {
    const timer = setInterval(() => {
      if (idxRef.current >= SEGMENTS.length) {
        clearInterval(timer);
        setThinking(false);
        return;
      }
      const seg = SEGMENTS[idxRef.current];
      idxRef.current += 1;
      setContent((c) => c + seg);
    }, 350);
    return () => clearInterval(timer);
  }, []);

  return (
    <Stack gap={3}>
      <ThinkingBlock content={content} thinking={thinking} duration={2} />
      <span className="text-xs text-tertiary-label">
        思考中自动展开并滚动到底，结束后自动收起标题
      </span>
    </Stack>
  );
}
