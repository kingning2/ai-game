import { Avatar, HStack, VStack } from '@agent-packages/shared-ui';

const sizes = ['xs', 'sm', 'md', 'lg'] as const;

export default function AvatarSize() {
  return (
    <HStack gap={5} align="end">
      {sizes.map((s) => (
        <VStack key={s} gap={1} align="center">
          <Avatar fallback="临" size={s} />
          <span className="text-xs text-tertiary-label">{s}</span>
        </VStack>
      ))}
    </HStack>
  );
}
