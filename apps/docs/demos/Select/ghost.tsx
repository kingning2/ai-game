import { useState } from 'react';
import { Select, Stack } from '@agent-packages/shared-ui';

const options = [
  { value: 'seed-5-pro', label: 'Seed 5 Pro' },
  { value: 'seed-5-lite', label: 'Seed 5 Lite', badge: '默认' },
  { value: 'seed-4', label: 'Seed 4' },
];

export default function SelectGhost() {
  const [value, setValue] = useState<string | null>('seed-5-lite');
  return (
    <Stack direction="row" gap={3} align="center">
      <Select
        variant="ghost"
        placement="top"
        value={value}
        onChange={setValue}
        options={options}
        ariaLabel="选择模型"
      />
      <span className="text-xs text-tertiary-label">ghost 变体 + 向上弹出</span>
    </Stack>
  );
}
