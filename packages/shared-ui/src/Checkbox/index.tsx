import type { ButtonHTMLAttributes } from 'react';
import { cn } from '../cn';

export type CheckboxProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'onChange' | 'value'> & {
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
};

/** 复选框 —— 勾选态 tint 底 + 白色对勾；按压 scale 反馈在 pointer-down 即时出现 */
export function Checkbox({ checked = false, onCheckedChange, className, ...props }: CheckboxProps) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={checked}
      onClick={() => onCheckedChange?.(!checked)}
      className={cn(
        'flex size-5 shrink-0 items-center justify-center rounded-md border',
        'transition duration-150 ease-out active:scale-90',
        'disabled:pointer-events-none disabled:opacity-40',
        checked
          ? 'border-tint bg-tint text-white'
          : 'border-separator bg-quaternary-fill text-transparent',
        className,
      )}
      {...props}
    >
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth={3.5}
        strokeLinecap="round"
        strokeLinejoin="round"
        className="size-3"
        aria-hidden
      >
        <path d="M5 13l4 4L19 7" />
      </svg>
    </button>
  );
}
