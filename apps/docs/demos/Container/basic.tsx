import { Container } from '@agent-packages/shared-ui';

export default function ContainerBasic() {
  return (
    <div className="rounded-xl border border-dashed border-separator p-2">
      <Container className="rounded-lg bg-tint/10 p-4 text-center text-sm text-secondary-label">
        水平居中（mx-auto）+ 响应式内边距（px-4 sm:px-6），宽度走断点令牌
      </Container>
    </div>
  );
}
