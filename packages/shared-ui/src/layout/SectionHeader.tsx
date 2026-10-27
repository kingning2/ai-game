import type { HTMLAttributes } from 'react';
import { cn } from '../cn';

export type SectionHeaderProps = HTMLAttributes<HTMLHeadingElement>;

/**
 * 分组标题 —— iOS 分组列表的 section header 规格：
 * 13px / semibold / 字距 +0.06em / 次要文字色。
 * 中文无大小写但字距同样生效，是「耐看的底层秩序」的一部分。
 */
export function SectionHeader({ className, ...props }: SectionHeaderProps) {
  return (
    <h2
      className={cn(
        'text-[13px] font-semibold tracking-[0.06em] text-secondary-label uppercase',
        className,
      )}
      {...props}
    />
  );
}
