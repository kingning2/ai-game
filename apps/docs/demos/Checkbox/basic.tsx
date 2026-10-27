import { useState } from 'react';
import { Checkbox, Stack } from '@agent-packages/shared-ui';

export default function CheckboxBasic() {
  const [checked, setChecked] = useState(false);
  return (
    <Stack direction="row" gap={2} align="center">
      <Checkbox checked={checked} onCheckedChange={setChecked} aria-label="示例复选框" />
      <span className="text-sm text-label">记住我</span>
      <span className="text-xs text-tertiary-label">当前：{checked ? '已勾选' : '未勾选'}</span>
    </Stack>
  );
}
