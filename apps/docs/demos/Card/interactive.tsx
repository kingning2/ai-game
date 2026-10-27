import { useState } from 'react';
import { Card, HStack, VStack } from '@agent-packages/shared-ui';

const plans = [
  { id: 'free', name: '免费版', desc: '每月 100 次调用' },
  { id: 'pro', name: '专业版', desc: '每月 10,000 次调用' },
];

export default function CardInteractive() {
  const [selected, setSelected] = useState('pro');
  return (
    <HStack gap={3} wrap role="radiogroup" aria-label="选择套餐">
      {plans.map((p) => (
        <Card
          key={p.id}
          role="radio"
          aria-checked={selected === p.id}
          onClick={() => setSelected(p.id)}
          className={`max-w-52 flex-1 cursor-pointer select-none transition ${
            selected === p.id ? 'ring-2 ring-tint' : 'hover:bg-tertiary-fill active:scale-[0.98]'
          }`}
        >
          <VStack gap={0.5}>
            <span className="text-sm font-semibold text-label">{p.name}</span>
            <span className="text-xs text-secondary-label">{p.desc}</span>
          </VStack>
        </Card>
      ))}
    </HStack>
  );
}
