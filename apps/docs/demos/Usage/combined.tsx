import { UsageBadge, UsageMeter } from '@agent-packages/shared-ui';
import { Card, Stack } from '@agent-packages/shared-ui';

/** 组合用法：消息卡片里徽章看单次消耗，进度条看窗口总量 */
export default function UsageCombined() {
  return (
    <Card className="w-full max-w-120">
      <Stack gap={4}>
        <div className="text-sm text-label">本次对话消耗</div>
        <UsageBadge prompt={1520} completion={8340} />
        <UsageMeter used={92400} total={200000} showPercent />
      </Stack>
    </Card>
  );
}
