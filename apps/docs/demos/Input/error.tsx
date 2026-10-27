import { Input, Stack } from '@agent-packages/shared-ui';

export default function InputError() {
  return (
    <Stack gap={3}>
      <Input aria-invalid placeholder="静默态：淡红描边提醒" defaultValue="not-an-email" />
      <Input aria-invalid placeholder="聚焦后：红色描边 + 红色光环" />
      <p className="text-xs text-tertiary-label">
        挂 <code className="text-xs">aria-invalid</code> 即生效，配合表单错误文案使用。
      </p>
    </Stack>
  );
}
