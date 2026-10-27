import { UsageBadge } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

/** prompt（↑ 输入）/ completion（↓ 输出）任意组合；两项都缺时渲染 null */
export default function UsageBadgeBasic() {
  return (
    <Stack direction="row" gap={3} align="center" wrap>
      <UsageBadge prompt={1520} completion={8340} />
      <UsageBadge prompt={92400} />
      <UsageBadge completion={1240000} />
      <UsageBadge />
      <span className="text-xs text-tertiary-label">← 两项都缺时整体不渲染</span>
    </Stack>
  );
}
