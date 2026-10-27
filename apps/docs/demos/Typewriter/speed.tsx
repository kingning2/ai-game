import { Typewriter } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const TEXT = '不同 speed 的节奏对比：每秒 20 / 60 / 120 个字符。';

export default function TypewriterSpeed() {
  return (
    <Stack gap={4}>
      <Stack gap={1}>
        <span className="text-xs text-tertiary-label">speed=20（缓慢，适合重要结论）</span>
        <Typewriter text={TEXT} speed={20} />
      </Stack>
      <Stack gap={1}>
        <span className="text-xs text-tertiary-label">speed=60（默认，流畅阅读节奏）</span>
        <Typewriter text={TEXT} speed={60} />
      </Stack>
      <Stack gap={1}>
        <span className="text-xs text-tertiary-label">speed=120（急速）</span>
        <Typewriter text={TEXT} speed={120} />
      </Stack>
    </Stack>
  );
}
