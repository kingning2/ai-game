import { useState } from 'react';
import { Slider, Stack } from '@agent-packages/shared-ui';

const SunIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    className="size-4 shrink-0 text-tertiary-label"
    aria-hidden
  >
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
  </svg>
);

export default function SliderWithIcons() {
  const [brightness, setBrightness] = useState(0.7);
  return (
    <Stack direction="row" gap={3} align="center">
      <SunIcon />
      <Slider
        value={brightness}
        min={0}
        max={1}
        step={0.01}
        onChange={setBrightness}
        aria-label="亮度"
      />
      <p className="w-10 shrink-0 text-end text-xs text-tertiary-label">
        {Math.round(brightness * 100)}%
      </p>
    </Stack>
  );
}
