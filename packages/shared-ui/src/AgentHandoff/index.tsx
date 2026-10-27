import { cn } from '../cn';

/*
 * AgentHandoff —— Agent 间交接回执
 *
 * 规则（scrimui agent-console）：
 * - 交接不是裸箭头：必须可审计 —— 下一个 agent 看到了什么（携带）、刻意不看什么（留下）、为什么交接
 * - 内联组件：渲染在时间线/消息流里，发生在哪就展示在哪，不开弹层
 * - 携带项用绿勾（已带入上下文），留下项用灰横线（明确排除）—— 两种视觉语义不许混用
 */

export interface AgentHandoffProps {
  /** 交接发起方名称 */
  from: string;
  /** 接收方名称 */
  to: string;
  /** 交接原因，如 '需要写作专家，不是再搜一轮' */
  reason?: string;
  /** 携带给下一棒的上下文清单 */
  carried?: string[];
  /** 刻意留下的内容清单（审计用） */
  withheld?: string[];
  /** 发生时间，如 '09:40:41' */
  time?: string;
  className?: string;
}

const ArrowIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3.5"
    aria-hidden
  >
    <path d="M5 12h14" />
    <path d="m13 6 6 6-6 6" />
  </svg>
);

const CheckIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.4"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3 shrink-0"
    aria-hidden
  >
    <path d="m5 12 5 5 9-10" />
  </svg>
);

const MinusIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.4"
    strokeLinecap="round"
    className="size-3 shrink-0"
    aria-hidden
  >
    <path d="M5 12h14" />
  </svg>
);

function ContextList({ items, mode }: { items: string[]; mode: 'carried' | 'withheld' }) {
  if (items.length === 0) return null;
  return (
    <div className="min-w-0 flex-1 basis-32">
      <div className="text-[11px] font-medium text-tertiary-label">
        {mode === 'carried' ? '携带上下文' : '刻意留下'}
      </div>
      <ul className="mt-1 space-y-1">
        {items.map((item, i) => (
          <li
            key={i}
            className="flex items-start gap-1.5 text-xs leading-snug text-secondary-label"
          >
            <span
              className={cn(
                'mt-0.5 flex size-3.5 shrink-0 items-center justify-center rounded-full',
                mode === 'carried'
                  ? 'bg-green/15 text-green'
                  : 'bg-tertiary-fill text-tertiary-label',
              )}
            >
              {mode === 'carried' ? <CheckIcon /> : <MinusIcon />}
            </span>
            <span className="break-words">{item}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

export function AgentHandoff({
  from,
  to,
  reason,
  carried = [],
  withheld = [],
  time,
  className,
}: AgentHandoffProps) {
  const initial = (s: string) => s.slice(0, 1).toUpperCase();

  return (
    <div
      className={cn(
        'overflow-hidden rounded-xl bg-secondary-grouped-background shadow-card',
        className,
      )}
    >
      {/* 头部：from → to 交接行 */}
      <div className="flex items-center gap-2 px-4 pb-2 pt-3.5">
        <span className="flex size-7 shrink-0 items-center justify-center rounded-full bg-tint/10 text-xs font-semibold text-tint">
          {initial(from)}
        </span>
        <span className="min-w-0 flex-1 truncate text-[13px] font-medium text-label">{from}</span>
        <span className="flex shrink-0 items-center gap-1 text-tertiary-label">
          <span className="h-px w-6 bg-separator" aria-hidden />
          <ArrowIcon />
          <span className="h-px w-6 bg-separator" aria-hidden />
        </span>
        <span className="flex size-7 shrink-0 items-center justify-center rounded-full bg-blue/15 text-xs font-semibold text-blue">
          {initial(to)}
        </span>
        <span className="min-w-0 flex-1 truncate text-[13px] font-medium text-label">{to}</span>
        {time && (
          <span className="shrink-0 text-[11px] tabular-nums text-tertiary-label">{time}</span>
        )}
      </div>

      {reason && (
        <div className="px-4 pb-2 text-xs leading-relaxed text-secondary-label">
          <span className="text-tertiary-label">原因 · </span>
          {reason}
        </div>
      )}

      {/* 上下文回执：携带 vs 留下 */}
      {(carried.length > 0 || withheld.length > 0) && (
        <div className="flex flex-wrap gap-x-6 gap-y-3 border-t border-separator/60 bg-tertiary-background/50 px-4 py-3">
          <ContextList items={carried} mode="carried" />
          <ContextList items={withheld} mode="withheld" />
        </div>
      )}
    </div>
  );
}
