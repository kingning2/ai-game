import type { ReactNode } from 'react';
import { cn } from '../cn';

const sizeClass = {
  sm: 'max-w-xl',
  md: 'max-w-3xl',
  lg: 'max-w-5xl',
} as const;

export type ContainerProps = {
  size?: keyof typeof sizeClass;
  className?: string;
  children?: ReactNode;
};

/** 页面内容容器：水平居中 + 响应式内边距，宽度走断点令牌而非写死 px */
export function Container({ size = 'md', className, children }: ContainerProps) {
  return (
    <div className={cn('mx-auto w-full px-4 sm:px-6', sizeClass[size], className)}>{children}</div>
  );
}
