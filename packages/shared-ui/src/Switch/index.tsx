import type { ButtonHTMLAttributes } from 'react';
import { cn } from '../cn';

export type SwitchProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'onChange' | 'value'> & {
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
};

/**
 * 开关 —— iOS UISwitch 规格：
 * - 开启态用 systemGreen（Apple 默认开关色），关闭态 fill 语义色
 * - 轨道 w-11 / 滑块 size-5 / 边距 0.5，位移正好滑满（20px），全部令牌值
 * - 滑块位移 spring 过冲回弹；rtl 下自动反向
 */
export function Switch({ checked = false, onCheckedChange, className, ...props }: SwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onCheckedChange?.(!checked)}
      className={cn(
        'relative h-6 w-11 shrink-0 rounded-full transition-colors duration-200 ease-out',
        checked ? 'bg-green' : 'bg-fill',
        'active:opacity-80 disabled:pointer-events-none disabled:opacity-40',
        className,
      )}
      {...props}
    >
      <span
        className={cn(
          'absolute start-0.5 top-0.5 size-5 rounded-full bg-white shadow-sm',
          'transition-transform duration-300 ease-spring',
          checked ? 'translate-x-5 rtl:-translate-x-5' : 'translate-x-0',
        )}
      />
    </button>
  );
}
