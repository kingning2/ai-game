import type { InputHTMLAttributes } from 'react';
import { cn } from '../cn';

export type InputProps = InputHTMLAttributes<HTMLInputElement>;

/**
 * 文本输入框 —— 描边式（C 端 web 主流预期）：
 * - 常态：白底 + 发丝 separator 描边，与卡片同色靠描边分层
 * - 聚焦：描边变 tint + 光环晕开 + 卡片阴影浮起，一次 200ms 过渡；白底不再加深
 * - 非法状态挂 aria-invalid：静默时淡红描边提醒，聚焦后换成红色光环，不再加线
 */
export function Input({ type = 'text', className, ...props }: InputProps) {
  return (
    <input
      type={type}
      className={cn(
        'h-9 w-full rounded-lg border border-separator bg-background px-3 text-sm text-label',
        'transition-all duration-200 ease-out',
        'placeholder:text-placeholder-text',
        'focus:border-tint/60 focus:shadow-card focus:ring-[3px] focus:ring-tint/10 focus:outline-none',
        'aria-invalid:border-destructive/50 focus:aria-invalid:border-destructive focus:aria-invalid:ring-destructive/10',
        'disabled:opacity-40',
        className,
      )}
      {...props}
    />
  );
}
