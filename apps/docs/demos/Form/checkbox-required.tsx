import { Button, CheckboxField, Form, Stack, useForm } from '@agent-packages/shared-ui';

export default function FormCheckboxRequired() {
  const form = useForm({
    defaultValues: { agree: false },
    onSubmit: async () => {},
  });
  return (
    <Form form={form}>
      <Stack gap={3}>
        <form.Field
          name="agree"
          validators={{ onChange: ({ value }) => (value ? undefined : '请先阅读并同意协议') }}
        >
          {(field) => (
            <CheckboxField
              field={field}
              label="我已阅读并同意《用户协议》"
              hint="未勾选时提交会被拦截"
            />
          )}
        </form.Field>
        <Button type="submit">注册</Button>
      </Stack>
    </Form>
  );
}
