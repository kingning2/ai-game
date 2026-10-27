import { Spinner } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

export default function SpinnerBasic() {
  return (
    <Stack direction="row" gap={5} align="center">
      <Spinner size="sm" />
      <Spinner size="md" />
      <Spinner size="lg" />
    </Stack>
  );
}
