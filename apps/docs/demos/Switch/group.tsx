import { useState } from 'react';
import { Stack, Switch } from '@agent-packages/shared-ui';

export default function SwitchGroup() {
  const [prefs, setPrefs] = useState({ mail: true, push: false, sms: false });
  return (
    <div className="rounded-2xl border border-separator bg-background p-4">
      <Stack gap={4}>
        {(
          [
            ['mail', '邮件通知', '账单与重要公告'],
            ['push', '推送通知', '实时活动提醒'],
            ['sms', '短信通知', '仅安全验证码'],
          ] as const
        ).map(([key, label, hint]) => (
          <div key={key} className="flex items-center justify-between gap-3">
            <div className="flex flex-col gap-0.5">
              <span className="text-sm font-medium text-label">{label}</span>
              <span className="text-xs text-tertiary-label">{hint}</span>
            </div>
            <Switch
              checked={prefs[key]}
              onCheckedChange={(v) => setPrefs((p) => ({ ...p, [key]: v }))}
              aria-label={label}
            />
          </div>
        ))}
      </Stack>
    </div>
  );
}
