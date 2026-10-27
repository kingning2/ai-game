import { Badge, HStack } from '@agent-packages/shared-ui';

export default function BadgeCustom() {
  return (
    <HStack gap={2} wrap align="center">
      <Badge className="px-2.5 py-1 text-sm">大号</Badge>
      <Badge className="rounded-md">方角</Badge>
      <Badge className="border border-separator">描边</Badge>
      <Badge tone="red" className="animate-pulse">
        闪烁
      </Badge>
    </HStack>
  );
}
