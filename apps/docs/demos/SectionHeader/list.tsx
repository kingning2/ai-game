import { Divider, SectionHeader, Stack } from '@agent-packages/shared-ui';

export default function SectionHeaderList() {
  return (
    <Stack gap={2}>
      <SectionHeader>通用</SectionHeader>
      <Stack className="overflow-hidden rounded-xl border border-separator">
        <p className="px-4 py-3 text-sm">关于本机</p>
        <Divider inset />
        <p className="px-4 py-3 text-sm">软件更新</p>
      </Stack>
      <SectionHeader>隐私</SectionHeader>
      <Stack className="overflow-hidden rounded-xl border border-separator">
        <p className="px-4 py-3 text-sm">定位服务</p>
        <Divider inset />
        <p className="px-4 py-3 text-sm">通讯录</p>
      </Stack>
    </Stack>
  );
}
