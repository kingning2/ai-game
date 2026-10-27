import { Button } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

export default function ButtonDisabled() {
  return (
    <Stack direction="row" spacing={3} wrap>
      <Button disabled>禁用主按钮</Button>
      <Button variant="tinted" disabled>
        禁用次按钮
      </Button>
    </Stack>
  );
}
