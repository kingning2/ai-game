import { Badge, Card, HStack, VStack } from '@agent-packages/shared-ui';

export default function BadgeStatus() {
  return (
    <VStack gap={4} className="max-w-md">
      <VStack gap={2}>
        <HStack justify="between" align="center">
          <span className="text-sm text-label">设计评审</span>
          <Badge tone="green">已通过</Badge>
        </HStack>
        <HStack justify="between" align="center">
          <span className="text-sm text-label">接口联调</span>
          <Badge tone="blue">进行中</Badge>
        </HStack>
        <HStack justify="between" align="center">
          <span className="text-sm text-label">回归测试</span>
          <Badge tone="red">已阻塞</Badge>
        </HStack>
      </VStack>
      <Card>
        <HStack align="center" gap={2}>
          <span className="text-sm font-semibold text-label">Agent 任务编排</span>
          <Badge tone="tint">Beta</Badge>
        </HStack>
        <p className="mt-1 text-xs text-secondary-label">把多步任务拆解成可复跑的工作流。</p>
      </Card>
    </VStack>
  );
}
