import { HStack, Stack } from '@agent-packages/shared-ui';

function Box({ label, className = '' }: { label: string; className?: string }) {
  return (
    <div
      className={`flex h-12 w-20 items-center justify-center rounded-lg bg-tint/10 px-3 text-sm text-secondary-label ${className}`}
    >
      {label}
    </div>
  );
}

export default function StackGap() {
  return (
    <HStack align="start" gap={8}>
      {([0, 3, 8] as const).map((g) => (
        <div key={g}>
          <p className="mb-2 text-xs text-tertiary-label">{`gap=${g}`}</p>
          <Stack gap={g}>
            <Box label="1" />
            <Box label="2" />
            <Box label="3" />
          </Stack>
        </div>
      ))}
    </HStack>
  );
}
