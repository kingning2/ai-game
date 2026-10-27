import { useState } from 'react';
import { Select, Stack } from '@agent-packages/shared-ui';

const options = [
  { value: 'balanced', label: '标准模式', badge: '默认' },
  { value: 'reasoning', label: '深度思考', badge: '推理' },
  { value: 'fast', label: '极速模式' },
  { value: 'legacy', label: '旧版模型', disabled: true },
];

export default function SelectBadge() {
  const [value, setValue] = useState<string | null>('balanced');
  return (
    <Stack gap={3}>
      <Select value={value} onChange={setValue} options={options} ariaLabel="选择模式" />
      <p className="text-xs text-tertiary-label">badge 渲染为浅色小字；disabled 项不可选中。</p>
    </Stack>
  );
}
