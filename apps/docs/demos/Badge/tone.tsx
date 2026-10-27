import { Badge, HStack } from '@agent-packages/shared-ui';

export default function BadgeTone() {
  return (
    <HStack gap={2} wrap align="center">
      <Badge tone="blue">进行中</Badge>
      <Badge tone="tint">推荐</Badge>
      <Badge tone="green">已完成</Badge>
      <Badge tone="orange">待审核</Badge>
      <Badge tone="red">已失败</Badge>
      <Badge tone="gray">草稿</Badge>
    </HStack>
  );
}
