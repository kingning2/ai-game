import { Container, Divider, SectionHeader, Stack } from '@agent-packages/shared-ui';

export default function ContainerPage() {
  return (
    <div className="rounded-xl border border-dashed border-separator p-2">
      <Container size="sm">
        <Stack gap={5} className="py-4">
          <SectionHeader>设置</SectionHeader>
          <Stack gap={3}>
            <p className="text-sm text-secondary-label">账户信息</p>
            <Divider />
            <p className="text-sm text-secondary-label">外观</p>
          </Stack>
          <SectionHeader>关于</SectionHeader>
          <p className="text-sm text-secondary-label">版本 1.0.0</p>
        </Stack>
      </Container>
    </div>
  );
}
