import { Divider, HStack } from '@agent-packages/shared-ui';

export default function DividerVertical() {
  return (
    <HStack gap={4}>
      <p className="text-sm text-secondary-label">左侧</p>
      <Divider vertical />
      <p className="text-sm text-secondary-label">中间</p>
      <Divider vertical />
      <p className="text-sm text-secondary-label">右侧</p>
    </HStack>
  );
}
