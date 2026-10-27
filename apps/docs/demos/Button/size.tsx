import { Button } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

export default function ButtonSize() {
  return (
    <Stack direction="row" spacing={3} align="center" wrap>
      <Button size="sm">小号</Button>
      <Button size="md">中号</Button>
      <Button size="lg">大号</Button>
    </Stack>
  );
}
