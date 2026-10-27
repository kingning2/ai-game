import { useState } from 'react';
import { Button, Form, Stack, TextField, useForm } from '@agent-packages/shared-ui';

function wait(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export default function FormAsyncSubmit() {
  const [sending, setSending] = useState(false);
  const [done, setDone] = useState(false);
  const form = useForm({
    defaultValues: { invite: '' },
    onSubmit: async ({ value }) => {
      setSending(true);
      await wait(1200); // 模拟请求
      setSending(false);
      setDone(true);
      form.reset();
      return value;
    },
  });
  return (
    <Form form={form}>
      <Stack gap={3}>
        <form.Field name="invite">
          {(field) => <TextField field={field} label="邀请码" placeholder="输入 6 位邀请码" />}
        </form.Field>
        <Button type="submit" disabled={sending}>
          {sending ? '提交中…' : '提交'}
        </Button>
        {done && !sending && <p className="text-xs text-tertiary-label">提交成功，表单已重置。</p>}
      </Stack>
    </Form>
  );
}
