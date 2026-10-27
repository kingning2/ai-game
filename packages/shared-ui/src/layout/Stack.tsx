import { createElement, type CSSProperties, type ElementType, type ReactNode } from 'react';
import { cn } from '../cn';

/** 间距阶梯：只允许设计令牌值，禁止任意数值（写死 px） */
export type StackGap = 0 | 0.5 | 1 | 1.5 | 2 | 3 | 4 | 5 | 6 | 8 | 10 | 12;

const gapClass: Record<StackGap, string> = {
  0: 'gap-0',
  0.5: 'gap-0.5',
  1: 'gap-1',
  1.5: 'gap-1.5',
  2: 'gap-2',
  3: 'gap-3',
  4: 'gap-4',
  5: 'gap-5',
  6: 'gap-6',
  8: 'gap-8',
  10: 'gap-10',
  12: 'gap-12',
};

const alignClass = {
  stretch: 'items-stretch',
  start: 'items-start',
  center: 'items-center',
  end: 'items-end',
  baseline: 'items-baseline',
} as const;

const justifyClass = {
  start: 'justify-start',
  center: 'justify-center',
  end: 'justify-end',
  between: 'justify-between',
  around: 'justify-around',
  evenly: 'justify-evenly',
} as const;

export type StackProps = {
  /** 排列方向，默认纵向 */
  direction?: 'row' | 'column';
  gap?: StackGap;
  align?: keyof typeof alignClass;
  justify?: keyof typeof justifyClass;
  wrap?: boolean;
  as?: ElementType;
  className?: string;
  style?: CSSProperties;
  children?: ReactNode;
};

/**
 * 弹性布局原语 —— 约束：一维布局一律用 flex（本组件），grid 只留给二维场景
 * 间距 / 对齐 / 分布全部映射到 Tailwind 令牌阶梯，不接受任意数值
 */
export function Stack({
  direction = 'column',
  gap = 2,
  align = 'stretch',
  justify = 'start',
  wrap = false,
  as = 'div',
  className,
  style,
  children,
}: StackProps) {
  return createElement(
    as,
    {
      className: cn(
        'flex',
        direction === 'row' ? 'flex-row' : 'flex-col',
        wrap && 'flex-wrap',
        gapClass[gap],
        alignClass[align],
        justifyClass[justify],
        className,
      ),
      style,
    },
    children,
  );
}

/** 横向堆叠 */
export function HStack(props: Omit<StackProps, 'direction'>) {
  return <Stack direction="row" {...props} />;
}

/** 纵向堆叠 */
export function VStack(props: Omit<StackProps, 'direction'>) {
  return <Stack direction="column" {...props} />;
}

/** 撑开剩余空间的弹性占位（配合 HStack / VStack 把兄弟元素推向两端） */
export function Spacer({ className }: { className?: string }) {
  return <div aria-hidden className={cn('flex-1', className)} />;
}
