import { Stack, Checkbox } from '@agent-packages/shared-ui';

export default function CheckboxDisabled() {
  return (
    <Stack direction="row" gap={6} align="center">
      <Checkbox disabled aria-label="禁用复选框（未勾选）" />
      <Checkbox disabled checked aria-label="禁用复选框（已勾选）" />
      <span className="text-xs text-tertiary-label">未勾选 / 已勾选两种禁用态</span>
    </Stack>
  );
}
