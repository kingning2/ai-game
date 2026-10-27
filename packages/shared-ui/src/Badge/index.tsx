import type { HTMLAttributes } from 'react';
import { cn } from '../cn';

export type BadgeTone = 'blue' | 'tint' | 'green' | 'orange' | 'red' | 'gray';

const toneClass: Record<BadgeTone, string> = {
  blue: 'bg-blue/10 text-blue',
  tint: 'bg-tint/10 text-tint',
  green: 'bg-green/10 text-green',
  orange: 'bg-orange/10 text-orange',
  red: 'bg-red/10 text-red',
  gray: 'bg-fill text-secondary-label',
};

export type BadgeProps = HTMLAttributes<HTMLSpanElement> & {
  tone?: BadgeTone;
};

/** 徽标：系统色 10% 透明度胶囊（Apple 着色徽章做法），状态语义一目了然 */
export function Badge({ tone = 'gray', className, ...props }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium whitespace-nowrap',
        toneClass[tone],
        className,
      )}
      {...props}
    />
  );
}
