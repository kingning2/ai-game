import { Spinner } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

/** 颜色跟随文字色：通过 className 传入任意文字色即可换色 */
export default function SpinnerColor() {
  return (
    <Stack direction="row" gap={5} align="center">
      <Spinner className="text-tint" />
      <Spinner className="text-green" />
      <Spinner className="text-orange" />
      <Spinner className="text-red" />
      <Spinner className="text-label" />
    </Stack>
  );
}
