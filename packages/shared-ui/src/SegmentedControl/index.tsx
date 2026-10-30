import { useRef, type CSSProperties, type KeyboardEvent } from 'react';
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
 * - WAI-ARIA radiogroup：roving tabindex（只有选中项可 Tab 到）+ 方向键切换并跟随焦点
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
  const optionRefs = useRef<(HTMLButtonElement | null)[]>([]);

  // 方向键在选项间移动并同步选中（循环），焦点跟随选中项
  const onGroupKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    const forward = e.key === 'ArrowRight' || e.key === 'ArrowDown';
    const backward = e.key === 'ArrowLeft' || e.key === 'ArrowUp';
    if (!forward && !backward) return;
    e.preventDefault();
    const delta = forward ? 1 : -1;
    const next = (selectedIndex + delta + options.length) % options.length;
    const option = options[next];
    if (!option) return;
    onChange(option.value);
    optionRefs.current[next]?.focus();
  };

  return (
    <div
      role="radiogroup"
      // 焦点交给内部 roving 的 radio 按钮（optionRefs），容器本身不进 Tab 序
      tabIndex={-1}
      onKeyDown={onGroupKeyDown}
      className={cn('relative inline-flex rounded-lg bg-fill p-0.5', className)}
      {...rest}
    >
      <span
        aria-hidden
        className={cn(
          'absolute inset-y-0.5 start-0.5 z-0 rounded-md bg-knob shadow-card',
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
      {options.map((option, index) => {
        const selected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            role="radio"
            aria-checked={selected}
            tabIndex={selected ? 0 : -1}
            ref={(el) => {
              optionRefs.current[index] = el;
            }}
            onClick={() => onChange(option.value)}
            className={cn(
              'relative z-10 flex-1 cursor-pointer rounded-md px-3 py-1 text-xs font-medium whitespace-nowrap',
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
