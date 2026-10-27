import { Stack } from '@agent-packages/shared-ui';

function Box({ label, className = '' }: { label: string; className?: string }) {
  return (
    <div
      className={`flex h-12 items-center justify-center rounded-lg bg-tint/10 px-3 text-sm text-secondary-label ${className}`}
    >
      {label}
    </div>
  );
}

export default function StackBasic() {
  return (
    <Stack gap={3}>
      <Box label="子元素 1" />
      <Box label="子元素 2" />
      <Box label="子元素 3" />
    </Stack>
  );
}
