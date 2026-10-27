import { Button, Form, Stack, TextField, useForm } from '@agent-packages/shared-ui';

export default function FormReset() {
  const form = useForm({
    defaultValues: { nickname: '小明' },
    onSubmit: async () => {},
  });
  return (
    <Form form={form}>
      <Stack gap={3}>
        <form.Field name="nickname">
          {(field) => <TextField field={field} label="昵称" hint="改动后点「重置」回到默认值" />}
        </form.Field>
        <Stack direction="row" gap={2}>
          <Button type="submit">提交</Button>
          <Button type="button" variant="gray" onClick={() => form.reset()}>
            重置
          </Button>
        </Stack>
      </Stack>
    </Form>
  );
}
