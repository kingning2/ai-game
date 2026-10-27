import { Button } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

export default function ButtonBasic() {
  return (
    <Stack direction="row" spacing={3} wrap>
      <Button>主要操作</Button>
      <Button variant="tinted">着色按钮</Button>
      <Button variant="plain">朴素按钮</Button>
      <Button variant="gray">灰色按钮</Button>
      <Button variant="destructive">危险操作</Button>
    </Stack>
  );
}
