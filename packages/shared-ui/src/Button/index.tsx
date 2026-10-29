import type { ButtonHTMLAttributes } from 'react';
import { cn } from '../cn';

export type ButtonVariant = 'solid' | 'tinted' | 'plain' | 'gray' | 'destructive';
export type ButtonSize = 'sm' | 'md' | 'lg';

const variantClass: Record<ButtonVariant, string> = {
  // Apple 规范：彩色填充（tint / destructive）上一律白字，dark 也不反转。
  // 光泽：白色微渐变叠在纯色底上（上亮下深，告别平面感）；悬停浮起 1px + 阴影，次级变体不浮起。
  solid:
    'bg-tint bg-gradient-to-b from-white/20 via-white/5 to-transparent text-on-accent hover:-translate-y-px hover:shadow-card active:translate-y-0 active:scale-[0.96]',
  tinted: 'bg-tint/10 text-tint hover:bg-tint/15 active:bg-tint/20',
  plain: 'text-tint hover:bg-tint/10 active:bg-tint/15',
  gray: 'bg-fill text-label hover:bg-secondary-fill hover:-translate-y-px hover:shadow-card active:translate-y-0 active:scale-[0.96]',
  destructive:
    'bg-destructive bg-gradient-to-b from-white/20 via-white/5 to-transparent text-on-accent hover:-translate-y-px hover:shadow-card active:translate-y-0 active:scale-[0.96]',
};

const sizeClass: Record<ButtonSize, string> = {
  sm: 'h-9 gap-1 px-3.5 text-[13px]',
  md: 'h-10 gap-1.5 px-5 text-sm',
  lg: 'h-12 gap-2 px-6 text-base',
};

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant;
  size?: ButtonSize;
};

/**
 * 按钮 —— Apple 规范：
 * - 语义令牌配色（tint / fill / destructive），明暗自适应；层次感靠透明度修饰符
 * - spring 曲线贯穿按压全程：按下即时缩放，松开带过冲回弹（iOS 手感核心）
 * - 高度 / 间距 / 字号全部走令牌阶梯，不写死数值
 * - 图标按钮（无可见文字）必须传 aria-label 说明用途
 */
export function Button({
  variant = 'solid',
  size = 'md',
  type = 'button',
  className,
  ...props
}: ButtonProps) {
  return (
    <button
      type={type}
      className={cn(
        'inline-flex cursor-pointer items-center justify-center rounded-xl font-medium whitespace-nowrap',
        'transition duration-300 ease-spring',
        'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-tint',
        'disabled:pointer-events-none disabled:opacity-40',
        variantClass[variant],
        sizeClass[size],
        className,
      )}
      {...props}
    />
  );
}
