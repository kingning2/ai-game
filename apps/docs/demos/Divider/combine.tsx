import { Divider, SectionHeader, Stack } from '@agent-packages/shared-ui';

export default function DividerCombine() {
  return (
    <Stack gap={2}>
      <SectionHeader>通用</SectionHeader>
      <Stack className="overflow-hidden rounded-xl border border-separator">
        <p className="px-4 py-3 text-sm">通知</p>
        <Divider inset />
        <p className="px-4 py-3 text-sm">声音与触感</p>
        <Divider inset />
        <p className="px-4 py-3 text-sm">勿扰模式</p>
      </Stack>
    </Stack>
  );
}
