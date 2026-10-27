import { Stack } from '@agent-packages/shared-ui';

function Box({ label, className = '' }: { label: string; className?: string }) {
  return (
    <div
      className={`flex w-20 items-center justify-center rounded-lg bg-tint/10 px-3 text-sm text-secondary-label ${className}`}
    >
      {label}
    </div>
  );
}

export default function StackAlign() {
  return (
    <Stack gap={6}>
      {(['start', 'center', 'end'] as const).map((a) => (
        <div key={a}>
          <p className="mb-2 text-xs text-tertiary-label">{`align="${a}"（交叉轴对齐，子元素高度不同时可见）`}</p>
          <Stack direction="row" align={a} gap={3}>
            <Box label="h-8" className="h-8" />
            <Box label="h-14" className="h-14" />
            <Box label="h-10" className="h-10" />
          </Stack>
        </div>
      ))}
      <div>
        <p className="mb-2 text-xs text-tertiary-label">{`justify="between"（主轴两端分布）`}</p>
        <Stack direction="row" justify="between" gap={3}>
          <Box label="起点" className="h-10" />
          <Box label="终点" className="h-10" />
        </Stack>
      </div>
    </Stack>
  );
}
