import { useEffect, useRef, useState, type ReactNode } from 'react';
import { cn } from '../cn';

/**
 * 下拉选择（受控）：
 * - outline：表单描边控件（与 Input 一致的圆角 / 聚焦 ring）
 * - ghost：轻量文字按钮（工具行内使用，如 Composer 的模型选择）
 * - 浮层菜单：placement 控制向下 / 向上弹出，点外部或 Esc 关闭
 */

export type SelectOption = {
  value: string;
  label: ReactNode;
  /** 可选标注（渲染为浅色小字），如「默认」「推理」 */
  badge?: ReactNode;
  disabled?: boolean;
};

export type SelectProps = {
  value: string | null;
  onChange?: (value: string) => void;
  options: SelectOption[];
  placeholder?: string;
  disabled?: boolean;
  /** outline = 表单描边控件；ghost = 轻量文字按钮，默认 outline */
  variant?: 'outline' | 'ghost';
  /** 浮层弹出方向，默认 bottom */
  placement?: 'bottom' | 'top';
  className?: string;
  ariaLabel?: string;
};

const ChevronDownIcon = ({ className }: { className?: string }) => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className={cn('size-3.5 shrink-0 transition-transform duration-200', className)}
    aria-hidden
  >
    <path d="M6 9l6 6 6-6" />
  </svg>
);

const CheckIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.5"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-4 shrink-0"
    aria-hidden
  >
    <path d="M20 6L9 17l-5-5" />
  </svg>
);

export function Select({
  value,
  onChange,
  options,
  placeholder = '请选择',
  disabled = false,
  variant = 'outline',
  placement = 'bottom',
  className,
  ariaLabel,
}: SelectProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const current = options.find((o) => o.value === value);

  // 点外部 / Esc 关闭
  useEffect(() => {
    if (!open) return;
    const close = (e: PointerEvent) => {
      if (!rootRef.current?.contains(e.target as Node)) setOpen(false);
    };
    const esc = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };
    document.addEventListener('pointerdown', close);
    document.addEventListener('keydown', esc);
    return () => {
      document.removeEventListener('pointerdown', close);
      document.removeEventListener('keydown', esc);
    };
  }, [open]);

  return (
    <div ref={rootRef} className={cn('relative', variant === 'outline' && 'w-full', className)}>
      <button
        type="button"
        disabled={disabled}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label={ariaLabel}
        onClick={() => setOpen((o) => !o)}
        className={cn(
          variant === 'outline'
            ? cn(
                'flex w-full items-center justify-between gap-2 rounded-xl border border-separator bg-background px-3 py-2 text-sm text-label',
                'transition-all duration-200 ease-out hover:border-label/30',
                open && 'border-tint/60 ring-[3px] ring-tint/10',
                disabled && 'pointer-events-none opacity-60',
              )
            : cn(
                'flex items-center gap-1 rounded-lg px-2 py-1.5 text-xs font-medium text-secondary-label transition-colors',
                'hover:bg-tertiary-fill hover:text-label',
                open && 'bg-tertiary-fill text-label',
                disabled && 'pointer-events-none opacity-60',
              ),
        )}
      >
        <span
          className={cn('truncate', variant === 'outline' && !current && 'text-placeholder-text')}
        >
          {current?.label ?? placeholder}
        </span>
        {current?.badge != null && (
          <span className="shrink-0 text-[11px] text-tertiary-label">{current.badge}</span>
        )}
        <ChevronDownIcon className={open ? 'rotate-180' : undefined} />
      </button>

      {open && (
        <ul
          role="listbox"
          aria-label={ariaLabel}
          className={cn(
            'absolute start-0 z-50 min-w-full rounded-xl bg-background p-1 shadow-popover',
            placement === 'top' ? 'bottom-full mb-2' : 'top-full mt-2',
          )}
        >
          {options.map((o) => (
            <li key={o.value}>
              <button
                type="button"
                role="option"
                aria-selected={o.value === value}
                disabled={o.disabled}
                onClick={() => {
                  onChange?.(o.value);
                  setOpen(false);
                }}
                className={cn(
                  'flex w-full items-center justify-between gap-3 rounded-lg px-2.5 py-1.5 text-start text-[13px] transition-colors',
                  'hover:bg-tertiary-fill disabled:pointer-events-none disabled:opacity-40',
                  o.value === value ? 'font-medium text-tint' : 'text-label',
                )}
              >
                <span className="flex items-center gap-1.5 truncate">
                  {o.label}
                  {o.badge != null && (
                    <span className="text-[11px] text-tertiary-label">{o.badge}</span>
                  )}
                </span>
                {o.value === value && <CheckIcon />}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
