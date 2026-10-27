import { formatTokens } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const CASES = [7, 999, 1234, 1200, 1000, 1520000, -3400];

/** formatTokens 纯函数：token 数格式化为 1.2K / 3.4M，去尾零 */
export default function UsageFormatTokens() {
  return (
    <Stack gap={1.5} className="w-fit font-mono text-xs tabular-nums">
      {CASES.map((n) => (
        <div key={n} className="flex gap-4">
          <span className="w-24 text-end text-secondary-label">{n}</span>
          <span className="text-tint">→ {formatTokens(n)}</span>
        </div>
      ))}
    </Stack>
  );
}
