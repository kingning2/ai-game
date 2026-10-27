import { useState } from 'react';
import { Button, Form, Stack, TextField, useForm } from '@agent-packages/shared-ui';

export default function FormValidation() {
  const [done, setDone] = useState(false);
  const form = useForm({
    defaultValues: { email: '', password: '' },
    onSubmit: async () => setDone(true),
  });
  return (
    <Form form={form}>
      <Stack gap={3}>
        <form.Field
          name="email"
          validators={{
            onChange: ({ value }) => (/^\S+@\S+\.\S+$/.test(value) ? undefined : '邮箱格式不正确'),
          }}
        >
          {(field) => (
            <TextField
              field={field}
              label="邮箱"
              placeholder="you@example.com"
              hint="用于登录与找回密码"
            />
          )}
        </form.Field>
        <form.Field
          name="password"
          validators={{
            onChange: ({ value }) => (value.length >= 8 ? undefined : '密码至少 8 位'),
          }}
        >
          {(field) => (
            <TextField field={field} label="密码" type="password" placeholder="至少 8 位" />
          )}
        </form.Field>
        <Button type="submit">登录</Button>
        {done && <p className="text-xs text-tertiary-label">校验通过，已提交。</p>}
      </Stack>
    </Form>
  );
}
