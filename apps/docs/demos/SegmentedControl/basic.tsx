import { useState } from 'react';
import { SegmentedControl, Stack } from '@agent-packages/shared-ui';

const options = [
  { value: 'day', label: '日' },
  { value: 'week', label: '周' },
  { value: 'month', label: '月' },
] as const;

type Period = (typeof options)[number]['value'];

export default function SegmentedBasic() {
  const [value, setValue] = useState<Period>('week');
  return (
    <Stack direction="row" gap={3} align="center">
      <SegmentedControl options={options} value={value} onChange={setValue} aria-label="统计周期" />
      <span className="text-xs text-tertiary-label">当前：{value}</span>
    </Stack>
  );
}
