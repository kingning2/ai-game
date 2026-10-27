import { Input, Stack } from '@agent-packages/shared-ui';

export default function InputDisabled() {
  return (
    <Stack gap={3}>
      <Input disabled placeholder="禁用（空值）" />
      <Input disabled defaultValue="只读内容" aria-label="禁用输入框" />
    </Stack>
  );
}
