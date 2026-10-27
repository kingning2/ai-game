import type { HTMLAttributes } from 'react';
import { cn } from '../cn';

export type CardProps = HTMLAttributes<HTMLDivElement>;

/**
 * 卡片 —— Apple 分组单元格样式：在 grouped 背景上使用，分隔与投影都走令牌；
 * 顶部一道 1px 内高光（light 下白底不可见、无害；dark 下深灰底上提亮顶缘，
 * 是 Apple 深色卡片「受光」的关键细节）
 */
export function Card({ className, ...props }: CardProps) {
  return (
    <div
      className={cn(
        'rounded-xl bg-secondary-grouped-background p-4 shadow-card shadow-[inset_0_1px_0_0_rgb(255_255_255/0.06)]',
        className,
      )}
      {...props}
    />
  );
}
