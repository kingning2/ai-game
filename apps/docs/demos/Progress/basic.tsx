import { Progress } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

export default function ProgressBasic() {
  return (
    <Stack gap={5}>
      <Progress value={0} />
      <Progress value={25} />
      <Progress value={60} />
      <Progress value={100} />
    </Stack>
  );
}
