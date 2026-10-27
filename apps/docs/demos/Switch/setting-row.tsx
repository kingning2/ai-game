import { useState } from 'react';
import { Stack, Switch } from '@agent-packages/shared-ui';

function SettingRow({
  label,
  hint,
  checked,
  onCheckedChange,
}: {
  label: string;
  hint: string;
  checked: boolean;
  onCheckedChange: (v: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between gap-3">
      <div className="flex flex-col gap-0.5">
        <span className="text-sm font-medium text-label">{label}</span>
        <span className="text-xs text-tertiary-label">{hint}</span>
      </div>
      <Switch checked={checked} onCheckedChange={onCheckedChange} aria-label={label} />
    </div>
  );
}

export default function SwitchSettingRow() {
  const [wifi, setWifi] = useState(true);
  const [dnd, setDnd] = useState(false);
  return (
    <div className="rounded-2xl border border-separator bg-background p-4">
      <Stack gap={4}>
        <SettingRow
          label="Wi-Fi"
          hint="连接后自动同步数据"
          checked={wifi}
          onCheckedChange={setWifi}
        />
        <SettingRow label="勿扰模式" hint="静音全部通知" checked={dnd} onCheckedChange={setDnd} />
      </Stack>
    </div>
  );
}
