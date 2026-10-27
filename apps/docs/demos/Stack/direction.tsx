import { Stack } from '@agent-packages/shared-ui';

function Box({ label, className = '' }: { label: string; className?: string }) {
  return (
    <div
      className={`flex items-center justify-center rounded-lg bg-tint/10 px-3 text-sm text-secondary-label ${className}`}
    >
      {label}
    </div>
  );
}

export default function StackDirection() {
  return (
    <Stack gap={6}>
      <div>
        <p className="mb-2 text-xs text-tertiary-label">{`direction="row"（横向排列，等价 HStack）`}</p>
        <Stack direction="row" gap={3}>
          <Box label="1" className="h-12 w-20" />
          <Box label="2" className="h-12 w-20" />
          <Box label="3" className="h-12 w-20" />
        </Stack>
      </div>
      <div>
        <p className="mb-2 text-xs text-tertiary-label">{`direction="column"（纵向排列，默认值，等价 VStack）`}</p>
        <Stack direction="column" gap={3}>
          <Box label="1" className="h-12" />
          <Box label="2" className="h-12" />
        </Stack>
      </div>
    </Stack>
  );
}
