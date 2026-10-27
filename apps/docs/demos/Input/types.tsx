import { Input, Stack } from '@agent-packages/shared-ui';

export default function InputTypes() {
  return (
    <Stack gap={3}>
      <Input type="password" placeholder="密码（type=password）" autoComplete="new-password" />
      <Input type="search" placeholder="搜索（type=search）" />
      <Input type="number" placeholder="数量（type=number）" min={0} />
    </Stack>
  );
}
