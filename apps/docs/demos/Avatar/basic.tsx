import { Avatar, HStack } from '@agent-packages/shared-ui';

export default function AvatarBasic() {
  return (
    <HStack gap={3} align="center">
      <Avatar fallback="临" />
      <Avatar fallback="周" />
      <Avatar fallback="AG" />
      <span className="text-xs text-tertiary-label">无图时渲染 fallback 缩写</span>
    </HStack>
  );
}
