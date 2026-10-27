import { useState } from 'react';
import { Input, Slider, Stack } from '@agent-packages/shared-ui';

export default function SliderSynced() {
  const [value, setValue] = useState(25);
  return (
    <Stack direction="row" gap={4} align="center">
      <Slider
        value={value}
        min={0}
        max={100}
        onChange={setValue}
        aria-label="与输入框联动"
        className="flex-1"
      />
      <Input
        type="number"
        min={0}
        max={100}
        value={value}
        onChange={(e) => {
          const next = Number(e.target.value);
          if (!Number.isNaN(next)) setValue(Math.min(100, Math.max(0, next)));
        }}
        className="w-20 shrink-0 text-center"
        aria-label="滑块数值输入"
      />
    </Stack>
  );
}
