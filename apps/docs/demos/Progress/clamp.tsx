import { Progress } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

/** value 超出 0–100 时自动钳制，不会溢出轨道 */
export default function ProgressClamp() {
  return (
    <Stack gap={5}>
      <Progress value={-20} />
      <Progress value={150} />
    </Stack>
  );
}
