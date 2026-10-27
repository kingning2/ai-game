import { useState } from 'react';
import { Checkbox, Stack } from '@agent-packages/shared-ui';

const ALL = ['React', 'Vue', 'Svelte'] as const;

export default function CheckboxGroup() {
  const [selected, setSelected] = useState<string[]>(['React']);
  const toggle = (item: string) =>
    setSelected((prev) => (prev.includes(item) ? prev.filter((v) => v !== item) : [...prev, item]));
  return (
    <Stack gap={2}>
      {ALL.map((item) => (
        <label key={item} className="flex cursor-pointer items-center gap-2">
          <Checkbox checked={selected.includes(item)} onCheckedChange={() => toggle(item)} />
          <span className="text-sm text-label">{item}</span>
        </label>
      ))}
      <p className="text-xs text-tertiary-label">已选：{selected.join('、') || '（无）'}</p>
    </Stack>
  );
}
