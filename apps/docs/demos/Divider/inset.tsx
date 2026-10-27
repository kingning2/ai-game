import { Divider, Stack } from '@agent-packages/shared-ui';

export default function DividerInset() {
  return (
    <Stack className="rounded-xl border border-separator p-3">
      <p className="py-2 text-sm text-secondary-label">普通分隔线（贴满整行）</p>
      <Divider />
      <p className="py-2 text-sm text-secondary-label">inset 分隔线（ms-4 缩进，iOS 列表样式）</p>
      <Divider inset />
      <p className="py-2 text-sm text-secondary-label">行尾内容</p>
    </Stack>
  );
}
