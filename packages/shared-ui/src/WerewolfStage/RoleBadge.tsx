import type { HTMLAttributes } from 'react';
import { cn } from '../cn';

/** 身份色调：与引擎 Camp 对齐——狼人红、好人黄 */
export type WerewolfRoleTone = 'wolf' | 'good';

const toneClass: Record<WerewolfRoleTone, string> = {
  wolf: 'bg-red text-on-accent',
  good: 'bg-yellow/90 text-stage-ink-strong',
};

export type RoleBadgeProps = HTMLAttributes<HTMLSpanElement> & {
  /** 身份文案：狼人 / 平民 / 预言家…（由调用方从角色档案下发） */
  label: string;
  tone?: WerewolfRoleTone;
};

/** 身份标签：竖排字，贴座位靠舞台中央一侧（上帝视角，引擎快照裁剪在调用方完成） */
export function RoleBadge({ label, tone = 'good', className, ...props }: RoleBadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex shrink-0 items-center justify-center rounded-md px-0.5 py-1.5',
        'text-xs font-medium leading-none tracking-widest',
        '[writing-mode:vertical-rl]',
        toneClass[tone],
        className,
      )}
      {...props}
    >
      {label}
    </span>
  );
}
