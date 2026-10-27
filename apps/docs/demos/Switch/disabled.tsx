import { Stack, Switch } from '@agent-packages/shared-ui';

export default function SwitchDisabled() {
  return (
    <Stack direction="row" gap={6} align="center">
      <Switch disabled aria-label="禁用开关（关）" />
      <Switch disabled checked aria-label="禁用开关（开）" />
      <span className="text-xs text-tertiary-label">关闭 / 开启两种禁用态</span>
    </Stack>
  );
}
