import { Divider, HStack, SectionHeader, Stack } from '@agent-packages/shared-ui';

export default function SectionHeaderForm() {
  return (
    <Stack gap={2}>
      <SectionHeader>账户</SectionHeader>
      <Stack className="rounded-xl border border-separator px-4">
        <HStack justify="between" align="center" className="py-3">
          <span className="text-sm">语言与地区</span>
          <span className="text-sm text-tertiary-label">简体中文</span>
        </HStack>
        <Divider inset />
        <HStack justify="between" align="center" className="py-3">
          <span className="text-sm">日期与时间</span>
          <span className="text-sm text-tertiary-label">自动设置</span>
        </HStack>
      </Stack>
    </Stack>
  );
}
