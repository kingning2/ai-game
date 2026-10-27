import { useState } from 'react';
import { Button, Form, Stack, TextField, useForm } from '@agent-packages/shared-ui';

export default function FormBasic() {
  const [submitted, setSubmitted] = useState<string | null>(null);
  const form = useForm({
    defaultValues: { nickname: '' },
    onSubmit: async ({ value }) => {
      setSubmitted(value.nickname);
    },
  });
  return (
    <Form form={form}>
      <Stack gap={3}>
        <form.Field name="nickname">
          {(field) => <TextField field={field} label="昵称" placeholder="给自己起个名字" />}
        </form.Field>
        <Button type="submit">提交</Button>
        {submitted && <p className="text-xs text-tertiary-label">已提交：{submitted}</p>}
      </Stack>
    </Form>
  );
}
