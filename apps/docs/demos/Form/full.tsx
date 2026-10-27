import { useState } from 'react';
import {
  Button,
  CheckboxField,
  Form,
  Stack,
  SwitchField,
  TextField,
  useForm,
} from '@agent-packages/shared-ui';

export default function FormFull() {
  const [done, setDone] = useState(false);
  const form = useForm({
    defaultValues: { nickname: '', email: '', subscribe: true, agree: false },
    onSubmit: async () => {
      setDone(true);
    },
  });
  return (
    <Form form={form} className="rounded-2xl border border-separator bg-background p-4">
      <Stack gap={4}>
        <form.Field
          name="nickname"
          validators={{ onChange: ({ value }) => (value.trim() ? undefined : '请输入昵称') }}
        >
          {(field) => <TextField field={field} label="昵称" placeholder="展示名称" />}
        </form.Field>
        <form.Field
          name="email"
          validators={{
            onChange: ({ value }) => (/^\S+@\S+\.\S+$/.test(value) ? undefined : '邮箱格式不正确'),
          }}
        >
          {(field) => <TextField field={field} label="邮箱" placeholder="you@example.com" />}
        </form.Field>
        <form.Field name="subscribe">
          {(field) => <SwitchField field={field} label="订阅产品动态" hint="每月一封，随时退订" />}
        </form.Field>
        <form.Field
          name="agree"
          validators={{ onChange: ({ value }) => (value ? undefined : '请先阅读并同意协议') }}
        >
          {(field) => <CheckboxField field={field} label="我已阅读并同意《用户协议》" />}
        </form.Field>
        <Button type="submit">完成注册</Button>
        {done && <p className="text-xs text-tertiary-label">注册成功，欢迎加入！</p>}
      </Stack>
    </Form>
  );
}
