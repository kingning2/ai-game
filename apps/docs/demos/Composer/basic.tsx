import { useState } from 'react';
import { Composer, Stack } from '@agent-packages/shared-ui';

export default function ComposerBasic() {
  const [value, setValue] = useState('');
  const [sent, setSent] = useState<string | null>(null);
  return (
    <Stack gap={3}>
      <Composer
        value={value}
        onChange={setValue}
        placeholder="输入消息…"
        onSend={() => {
          setSent(value);
          setValue('');
        }}
      />
      {sent && <p className="text-xs text-tertiary-label">已发送：{sent}</p>}
    </Stack>
  );
}
