import { Avatar, Badge, Card, HStack, VStack } from '@agent-packages/shared-ui';

export default function CardProfile() {
  return (
    <Card className="max-w-md">
      <HStack gap={3} align="center">
        <Avatar fallback="临" size="lg" />
        <VStack gap={0.5} className="min-w-0 flex-1">
          <HStack gap={2} align="center">
            <span className="text-sm font-semibold text-label">林小临</span>
            <Badge tone="tint">管理员</Badge>
          </HStack>
          <span className="truncate text-xs text-secondary-label">lin@example.com</span>
        </VStack>
      </HStack>
      <p className="mt-3 text-xs text-secondary-label">负责组件库的设计令牌与无障碍支持。</p>
    </Card>
  );
}
