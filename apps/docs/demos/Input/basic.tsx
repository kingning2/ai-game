import { useState } from 'react';
import { Input, Stack } from '@agent-packages/shared-ui';

export default function InputBasic() {
  const [value, setValue] = useState('');
  return (
    <Stack gap={3}>
      <Input placeholder="请输入昵称" value={value} onChange={(e) => setValue(e.target.value)} />
      <p className="text-xs text-tertiary-label">当前值：{value || '（空）'}</p>
    </Stack>
  );
}
