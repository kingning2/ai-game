import { useState } from 'react';
import { Select, Stack } from '@agent-packages/shared-ui';

const options = [
  { value: 'react', label: 'React' },
  { value: 'vue', label: 'Vue' },
  { value: 'svelte', label: 'Svelte' },
  { value: 'solid', label: 'Solid' },
];

export default function SelectBasic() {
  const [value, setValue] = useState<string | null>('react');
  return (
    <Stack gap={3}>
      <Select value={value} onChange={setValue} options={options} ariaLabel="选择框架" />
      <p className="text-xs text-tertiary-label">当前值：{value ?? '（未选择）'}</p>
    </Stack>
  );
}
