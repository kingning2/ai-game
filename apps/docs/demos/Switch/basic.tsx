import { useState } from 'react';
import { Stack, Switch } from '@agent-packages/shared-ui';

export default function SwitchBasic() {
  const [checked, setChecked] = useState(false);
  return (
    <Stack direction="row" gap={3} align="center">
      <Switch checked={checked} onCheckedChange={setChecked} aria-label="示例开关" />
      <p className="text-xs text-tertiary-label">当前状态：{checked ? '开启' : '关闭'}</p>
    </Stack>
  );
}
