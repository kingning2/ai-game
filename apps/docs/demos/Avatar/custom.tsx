import { Avatar, HStack } from '@agent-packages/shared-ui';

export default function AvatarCustom() {
  return (
    <HStack gap={3} align="center">
      <Avatar fallback="方" className="rounded-lg" />
      <Avatar fallback="描" className="border-2 border-background shadow-card" />
      <Avatar fallback="绿" className="bg-green/15 text-green" />
      <Avatar fallback="+8" className="bg-tertiary-fill text-secondary-label" />
    </HStack>
  );
}
