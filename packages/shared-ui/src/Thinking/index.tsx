import { useEffect, useRef, useState } from 'react';
import { cn } from '../cn';
import { Spinner } from '../Spinner';

/**
 * 思考过程（AI 思维链面板，DeepSeek R1 / Claude thinking 风格）：
 * - 标题行可点击折叠 / 展开：思考中显示脉动「思考中…」，结束显示「已深度思考（用时 N 秒）」
 * - 内容区：左侧竖线 + 弱色小字，流式追加自动滚到底
 * - autoCollapse：思考中自动展开、结束自动收起（受控模式除外）
 * - 折叠动画用 grid-rows 0fr→1fr，高度自适应且顺滑
 */

export type ThinkingBlockProps = {
  /** 思维链内容（可流式追加，保留换行） */
  content: string;
  /** 思考进行中 */
  thinking?: boolean;
  /** 思考用时（秒）；thinking 结束后标题显示「已深度思考（用时 N 秒）」 */
  duration?: number;
  /** 结束态标题，默认「已深度思考」，设置了 duration 会附带用时 */
  label?: string;
  /** 手动展开 / 收起覆盖自动行为；thinking 重新开始时重置（非受控模式） */
  open?: boolean;
  onToggle?: (open: boolean) => void;
  className?: string;
};

const ChevronIcon = ({ className }: { className?: string }) => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className={cn('size-3.5 shrink-0 transition-transform duration-300', className)}
    aria-hidden
  >
    <path d="M6 9l6 6 6-6" />
  </svg>
);

export function ThinkingBlock({
  content,
  thinking = false,
  duration,
  label = '已深度思考',
  open: openProp,
  onToggle,
  className,
}: ThinkingBlockProps) {
  const controlled = openProp !== undefined;
  const [override, setOverride] = useState<boolean | null>(null);
  const [prevThinking, setPrevThinking] = useState(thinking);
  // thinking 上升沿重置手动覆盖（官方 render 期间派生模式，替代 effect 同步）
  if (prevThinking !== thinking) {
    setPrevThinking(thinking);
    setOverride(null);
  }
  const open = controlled ? openProp : (override ?? thinking);
  const bodyRef = useRef<HTMLDivElement>(null);

  const toggle = () => {
    const next = !open;
    if (!controlled) setOverride(next);
    onToggle?.(next);
  };

  // 思考中内容流式追加 → 滚到底
  useEffect(() => {
    if (!thinking || !open) return;
    const el = bodyRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [content, thinking, open]);

  const title = thinking ? (
    <span className="flex items-center gap-2">
      <Spinner size="sm" />
      <span className="animate-pulse">思考中…</span>
    </span>
  ) : (
    <span>
      {label}
      {duration != null && `（用时 ${duration} 秒）`}
    </span>
  );

  return (
    <div className={cn('flex flex-col', className)}>
      <button
        type="button"
        onClick={toggle}
        aria-expanded={open}
        className={cn(
          'flex w-fit items-center gap-1.5 rounded-lg py-1 pe-2 text-xs font-medium transition-colors',
          thinking ? 'text-secondary-label' : 'text-tertiary-label hover:text-label',
        )}
      >
        <ChevronIcon className={open ? 'rotate-180' : undefined} />
        {title}
      </button>

      <div
        className={cn(
          'grid transition-[grid-template-rows] duration-300 ease-out',
          open ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]',
        )}
      >
        <div className="overflow-hidden">
          <div
            ref={bodyRef}
            className={cn(
              'mt-0.5 max-h-64 overflow-y-auto border-s-2 border-separator ps-3',
              'whitespace-pre-wrap text-xs leading-relaxed text-tertiary-label',
            )}
          >
            {content}
          </div>
        </div>
      </div>
    </div>
  );
}
