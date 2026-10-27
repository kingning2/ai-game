import { Button, Spinner } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

/** 按钮加载中：Spinner + disabled 组合表达，而非内置 loading 属性 */
export default function SpinnerInline() {
  return (
    <Stack direction="row" gap={4} align="center">
      <Button variant="tinted" disabled>
        <Stack direction="row" gap={2} align="center" as="span">
          <Spinner size="sm" className="text-tint" />
          保存中
        </Stack>
      </Button>
      <Button variant="gray" disabled>
        <Stack direction="row" gap={2} align="center" as="span">
          <Spinner size="sm" className="text-tertiary-label" />
          加载中
        </Stack>
      </Button>
    </Stack>
  );
}
