import { type ReactNode } from 'react';
import { useForm, type AnyFieldApi } from '@tanstack/react-form';
import { Checkbox } from '../Checkbox';
import { Input } from '../Input';
import { Switch } from '../Switch';
import { cn } from '../cn';

/**
 * 表单 —— TanStack Form（headless）+ 现有组件绑定：
 * - useForm 由 TanStack 提供（此处 re-export），校验/提交/订阅状态全部 headless
 * - TextField / SwitchField / CheckboxField：负责把 field 绑到自家组件上，
 *   统一 label、hint、错误文案（text-destructive + aria-invalid 红描边联动）
 * - <Form> 薄壳：接管 preventDefault + handleSubmit
 * 用法：
 *   const form = useForm({ defaultValues: {...}, onSubmit: async ({ value }) => {} });
 *   <Form form={form}>
 *     <form.Field name="email" validators={{ onChange: ({ value }) => isEmail(value) ? undefined : '邮箱格式不正确' }}>
 *       {(field) => <TextField field={field} label="邮箱" />}
 *     </form.Field>
 *   </Form>
 */

export { useForm };

/** 从 meta.errors（string | StandardSchemaV1Issue 混合数组）取出第一条可读错误 */
function firstError(errors: unknown[]): string | undefined {
  for (const e of errors) {
    if (typeof e === 'string') return e;
    if (e && typeof e === 'object' && 'message' in e) {
      const m = (e as { message: unknown }).message;
      if (typeof m === 'string') return m;
    }
  }
  return undefined;
}

function FieldText({ hint, error }: { hint?: string; error?: string }) {
  if (error) return <span className="text-xs text-destructive">{error}</span>;
  if (hint) return <span className="text-xs text-tertiary-label">{hint}</span>;
  return null;
}

/** 文本输入字段：label + Input + hint/错误文案 */
export function TextField({
  field,
  label,
  hint,
  placeholder,
  type = 'text',
  className,
}: {
  field: AnyFieldApi;
  label?: string;
  hint?: string;
  placeholder?: string;
  type?: string;
  className?: string;
}) {
  const error = firstError(field.state.meta.errors);
  return (
    <div className={cn('flex flex-col gap-1.5', className)}>
      {label && <span className="text-sm font-medium text-label">{label}</span>}
      <Input
        type={type}
        value={String(field.state.value ?? '')}
        placeholder={placeholder}
        aria-invalid={!!error}
        onChange={(e) => field.handleChange(e.target.value)}
        onBlur={field.handleBlur}
      />
      <FieldText hint={hint} error={error} />
    </div>
  );
}

/** 开关字段：iOS 设置风格 —— 左 label/hint，右 Switch */
export function SwitchField({
  field,
  label,
  hint,
  className,
}: {
  field: AnyFieldApi;
  label?: string;
  hint?: string;
  className?: string;
}) {
  const error = firstError(field.state.meta.errors);
  return (
    <div className={cn('flex items-center justify-between gap-3', className)}>
      <div className="flex flex-col gap-0.5">
        {label && <span className="text-sm font-medium text-label">{label}</span>}
        {hint && <span className="text-xs text-tertiary-label">{hint}</span>}
        {error && <span className="text-xs text-destructive">{error}</span>}
      </div>
      <Switch checked={!!field.state.value} onCheckedChange={(v) => field.handleChange(v)} />
    </div>
  );
}

/** 勾选字段：横向 label + Checkbox */
export function CheckboxField({
  field,
  label,
  hint,
  className,
}: {
  field: AnyFieldApi;
  label?: string;
  hint?: string;
  className?: string;
}) {
  const error = firstError(field.state.meta.errors);
  return (
    <div className={cn('flex items-start gap-2.5', className)}>
      <Checkbox
        checked={!!field.state.value}
        onCheckedChange={(v) => field.handleChange(v)}
        className="mt-0.5"
      />
      <div className="flex flex-col gap-0.5">
        {label && <span className="text-sm font-medium text-label">{label}</span>}
        {hint && <span className="text-xs text-tertiary-label">{hint}</span>}
        {error && <span className="text-xs text-destructive">{error}</span>}
      </div>
    </div>
  );
}

type FormLike = { handleSubmit: () => Promise<void> | void };

/** 表单薄壳：接管 preventDefault + handleSubmit */
export function Form({
  form,
  className,
  children,
}: {
  form: FormLike;
  className?: string;
  children: ReactNode;
}) {
  return (
    <form
      className={className}
      noValidate
      onSubmit={(e) => {
        e.preventDefault();
        void form.handleSubmit();
      }}
    >
      {children}
    </form>
  );
}
