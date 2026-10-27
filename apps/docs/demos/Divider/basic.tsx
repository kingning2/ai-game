import { Divider, Stack } from '@agent-packages/shared-ui';

export default function DividerBasic() {
  return (
    <Stack gap={3}>
      <p className="text-sm text-secondary-label">上方内容</p>
      <Divider />
      <p className="text-sm text-secondary-label">下方内容</p>
    </Stack>
  );
}
