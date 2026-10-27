import type { HTMLAttributes } from 'react';
import { cn } from '../cn';

const sizeClass = {
  xs: 'size-6 text-xs',
  sm: 'size-8 text-xs',
  md: 'size-10 text-sm',
  lg: 'size-12 text-base',
} as const;

export type AvatarProps = HTMLAttributes<HTMLSpanElement> & {
  src?: string;
  alt?: string;
  /** 无图片时的回退缩写（一般取姓名首字母 / 末字） */
  fallback?: string;
  size?: keyof typeof sizeClass;
};

/** 头像：圆形，图片或 tint 回退缩写 */
export function Avatar({ src, alt, fallback, size = 'md', className, ...props }: AvatarProps) {
  return (
    <span
      className={cn(
        'relative inline-flex shrink-0 select-none items-center justify-center overflow-hidden rounded-full',
        'bg-tint/10 font-medium text-tint',
        sizeClass[size],
        className,
      )}
      {...props}
    >
      {src ? <img src={src} alt={alt ?? ''} className="size-full object-cover" /> : fallback}
    </span>
  );
}
