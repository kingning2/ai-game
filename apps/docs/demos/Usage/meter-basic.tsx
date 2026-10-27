import { UsageMeter } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

/** 三档色阶：<70% 主题色，70–90% 橙色，≥90% 红色 */
export default function UsageMeterBasic() {
  return (
    <Stack gap={5} className="w-full max-w-120">
      <UsageMeter used={48000} total={200000} />
      <UsageMeter used={150000} total={200000} />
      <UsageMeter used={190000} total={200000} />
    </Stack>
  );
}
