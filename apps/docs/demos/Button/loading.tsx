import { useState } from 'react';
import { Button, Spinner } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

export default function ButtonLoading() {
  const [saving, setSaving] = useState(false);

  return (
    <Stack direction="row" spacing={3} wrap>
      {/* 组件库遵循 Apple HIG：加载态不内置，用 Spinner + disabled 组合表达 */}
      <Button
        disabled={saving}
        onClick={() => {
          setSaving(true);
          window.setTimeout(() => setSaving(false), 2000);
        }}
      >
        {saving && <Spinner size="sm" />}
        {saving ? '保存中…' : '点击保存'}
      </Button>
    </Stack>
  );
}
