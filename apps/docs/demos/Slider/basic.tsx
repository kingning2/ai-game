import { useState } from 'react';
import { Slider, Stack } from '@agent-packages/shared-ui';

export default function SliderBasic() {
  const [value, setValue] = useState(40);
  return (
    <Stack gap={2}>
      <Slider value={value} onChange={setValue} aria-label="基础滑块" />
      <p className="text-xs text-tertiary-label">当前值：{value}</p>
    </Stack>
  );
}
