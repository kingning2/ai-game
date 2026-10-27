import { useState } from 'react';
import { Slider, Stack } from '@agent-packages/shared-ui';

export default function SliderStep() {
  const [percent, setPercent] = useState(60);
  const [opacity, setOpacity] = useState(0.5);
  return (
    <Stack gap={4}>
      <div className="flex flex-col gap-1.5">
        <span className="text-sm font-medium text-label">透明度（step=0.1）</span>
        <Slider
          value={opacity}
          min={0}
          max={1}
          step={0.1}
          onChange={setOpacity}
          aria-label="透明度"
        />
        <p className="text-xs text-tertiary-label">当前值：{opacity.toFixed(1)}</p>
      </div>
      <div className="flex flex-col gap-1.5">
        <span className="text-sm font-medium text-label">音量（step=10）</span>
        <Slider
          value={percent}
          min={0}
          max={100}
          step={10}
          onChange={setPercent}
          aria-label="音量"
        />
        <p className="text-xs text-tertiary-label">当前值：{percent}%</p>
      </div>
    </Stack>
  );
}
