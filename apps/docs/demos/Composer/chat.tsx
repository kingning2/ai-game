import { useRef, useState } from 'react';
import { Composer, Stack } from '@agent-packages/shared-ui';

type Message = { id: number; role: 'user' | 'assistant'; text: string };

const models = [
  { id: 'seed-5-pro', name: 'Seed 5 Pro' },
  { id: 'seed-5-lite', name: 'Seed 5 Lite', badge: '默认' },
];

export default function ComposerChat() {
  const [value, setValue] = useState('');
  const [model, setModel] = useState<string | undefined>('seed-5-lite');
  const [messages, setMessages] = useState<Message[]>([]);
  const [streaming, setStreaming] = useState(false);
  const nextId = useRef(1);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const reply = (question: string) => {
    setStreaming(true);
    timer.current = setTimeout(() => {
      setMessages((prev) => [
        ...prev,
        {
          id: nextId.current++,
          role: 'assistant',
          text: `已收到你的问题：「${question}」，这是模拟回复。`,
        },
      ]);
      setStreaming(false);
    }, 1500);
  };

  return (
    <Stack gap={3}>
      {messages.length > 0 && (
        <Stack gap={2}>
          {messages.map((m) => (
            <div key={m.id} className={m.role === 'user' ? 'text-end' : 'text-start'}>
              <span className="inline-block max-w-[85%] rounded-xl bg-tertiary-fill px-3 py-1.5 text-sm text-label">
                {m.text}
              </span>
            </div>
          ))}
        </Stack>
      )}
      <Composer
        value={value}
        onChange={setValue}
        models={models}
        model={model}
        onModelChange={setModel}
        streaming={streaming}
        placeholder={streaming ? '生成中…' : '输入消息，Enter 发送'}
        onSend={() => {
          const question = value.trim();
          if (!question) return;
          setMessages((prev) => [...prev, { id: nextId.current++, role: 'user', text: question }]);
          setValue('');
          reply(question);
        }}
        onStop={() => {
          if (timer.current) clearTimeout(timer.current);
          setStreaming(false);
        }}
      />
    </Stack>
  );
}
