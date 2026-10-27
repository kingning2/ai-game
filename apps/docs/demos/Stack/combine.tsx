import { Divider, HStack, Spacer, Stack, VStack } from '@agent-packages/shared-ui';

function Box({ label, className = '' }: { label: string; className?: string }) {
  return (
    <div
      className={`flex items-center justify-center rounded-lg bg-tint/10 px-3 text-sm text-secondary-label ${className}`}
    >
      {label}
    </div>
  );
}

export default function StackCombine() {
  return (
    <Stack gap={4} className="rounded-xl border border-separator p-4">
      <HStack align="center" gap={3}>
        <Box label="标题" className="h-9" />
        <Spacer />
        <Box label="操作 A" className="h-9" />
        <Box label="操作 B" className="h-9" />
      </HStack>
      <Divider />
      <VStack gap={2}>
        <Box label="内容区（VStack + Spacer 把操作推到行尾）" className="h-16" />
      </VStack>
    </Stack>
  );
}
