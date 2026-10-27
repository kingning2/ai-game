import {
  Button,
  CheckboxField,
  Form,
  Stack,
  SwitchField,
  TextField,
  useForm,
} from '@agent-packages/shared-ui';

export default function FormFields() {
  const form = useForm({
    defaultValues: { repo: '', autoSync: true, changelog: false },
    onSubmit: async () => {},
  });
  return (
    <Form form={form}>
      <Stack gap={4}>
        <form.Field name="repo">
          {(field) => (
            <TextField
              field={field}
              label="仓库名"
              hint="格式：owner/repo"
              placeholder="acme/web"
            />
          )}
        </form.Field>
        <form.Field name="autoSync">
          {(field) => <SwitchField field={field} label="自动同步" hint="每次提交后自动拉取变更" />}
        </form.Field>
        <form.Field name="changelog">
          {(field) => (
            <CheckboxField field={field} label="生成变更日志" hint="同步时自动附带 changelog" />
          )}
        </form.Field>
        <Button type="submit">保存</Button>
      </Stack>
    </Form>
  );
}
