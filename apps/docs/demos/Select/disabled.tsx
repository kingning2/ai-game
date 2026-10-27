import { Select, Stack } from '@agent-packages/shared-ui';

export default function SelectDisabled() {
  return (
    <Stack gap={3}>
      <Select
        value={null}
        options={[]}
        disabled
        placeholder="禁用（未选择）"
        ariaLabel="禁用选择器"
      />
      <Select
        value="react"
        options={[{ value: 'react', label: 'React' }]}
        disabled
        ariaLabel="禁用选择器（有值）"
      />
    </Stack>
  );
}
