import type { CSSProperties } from 'react';
import { cn } from '../cn';

export type SegmentedOption<T extends string> = {
  value: T;
  label: string;
};

export type SegmentedControlProps<T extends string> = {
  options: ReadonlyArray<SegmentedOption<T>>;
  value: T;
  onChange: (value: T) => void;
  'aria-label'?: string;
  className?: string;
};

/**
 * 分段控件 —— Apple 规格（滑动 thumb 版）：
 * - 等宽分段（flex-1，一维布局走 flex）
 * - 选中态是一个绝对定位的白底 thumb，随选择 spring 滑动（过冲回弹，iOS 手感核心），
 *   不再是每个选项各自切换背景的跳变
 * - thumb 位移走 CSS 变量 + rtl 变体，方向自适应；文字颜色仍即时切换
 */
export function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
  className,
  ...rest
}: SegmentedControlProps<T>) {
  const selectedIndex = Math.max(
    0,
    options.findIndex((option) => option.value === value),
  );

  return (
    <div
      role="radiogroup"
      className={cn('relative inline-flex rounded-lg bg-fill p-0.5', className)}
      {...rest}
    >
      <span
        aria-hidden
        className={cn(
          'absolute inset-y-0.5 start-0.5 z-0 rounded-md bg-secondary-grouped-background shadow-sm',
          'transition-transform duration-300 ease-spring',
          'translate-x-[calc(var(--seg-index)*100%)] rtl:-translate-x-[calc(var(--seg-index)*100%)]',
        )}
        style={
          {
            width: `calc((100% - 4px) / ${options.length})`,
            '--seg-index': selectedIndex,
          } as CSSProperties
        }
      />
      {options.map((option) => {
        const selected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            role="radio"
            aria-checked={selected}
            onClick={() => onChange(option.value)}
            className={cn(
              'relative z-10 flex-1 rounded-md px-3 py-1 text-xs font-medium whitespace-nowrap',
              'transition-colors duration-200 ease-out active:scale-[0.97]',
              selected ? 'text-label' : 'text-secondary-label hover:text-label',
            )}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
