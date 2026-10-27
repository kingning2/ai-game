import { Container, Stack } from '@agent-packages/shared-ui';

const sizeMaxWidth = {
  sm: 'max-w-xl（576px）',
  md: 'max-w-3xl（768px）',
  lg: 'max-w-5xl（1024px）',
};

export default function ContainerSize() {
  return (
    <Stack gap={4}>
      {(['sm', 'md', 'lg'] as const).map((s) => (
        <div key={s} className="rounded-xl border border-dashed border-separator p-2">
          <p className="mb-2 text-xs text-tertiary-label">{`size="${s}" → ${sizeMaxWidth[s]}`}</p>
          <Container
            size={s}
            className="rounded-lg bg-tint/10 p-3 text-center text-sm text-secondary-label"
          >
            {sizeMaxWidth[s]}
          </Container>
        </div>
      ))}
    </Stack>
  );
}
