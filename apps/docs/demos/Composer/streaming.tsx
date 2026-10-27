import { useRef, useState } from 'react';
import { Composer } from '@agent-packages/shared-ui';

export default function ComposerStreaming() {
  const [value, setValue] = useState('');
  const [streaming, setStreaming] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  return (
    <Composer
      value={value}
      onChange={setValue}
      streaming={streaming}
      placeholder={streaming ? '生成中…' : '输入消息…'}
      onSend={() => {
        setValue('');
        setStreaming(true);
        timer.current = setTimeout(() => setStreaming(false), 4000); // 模拟流式生成
      }}
      onStop={() => {
        if (timer.current) clearTimeout(timer.current);
        setStreaming(false);
      }}
    />
  );
}
