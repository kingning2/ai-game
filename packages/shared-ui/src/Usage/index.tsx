import { cn } from '../cn';

/**
 * 用量显示组件：
 * - formatTokens：token 数格式化（1.2K / 3.4M，等宽字体适用）
 * - UsageMeter：上下文窗口占用条（已用 / 总量 + 百分比，70% 转橙、90% 转红）
 * - UsageBadge：用量徽章（↑ 输入 · ↓ 输出，胶囊形等宽数字）
 */

/** 1234 → "1.2K"；去尾零（1200 → "1.2K"，1000 → "1K"） */
export function formatTokens(n: number): string {
  const fmt = (v: number, unit: string) => {
    const s = v >= 100 ? Math.round(v).toString() : v.toFixed(1);
    return `${s.endsWith('.0') ? s.slice(0, -2) : s}${unit}`;
  };
  if (Math.abs(n) >= 1_000_000) return fmt(n / 1_000_000, 'M');
  if (Math.abs(n) >= 1000) return fmt(n / 1000, 'K');
  return String(n);
}

export type UsageMeterProps = {
  /** 已用 tokens */
  used: number;
  /** 上下文窗口总量 */
  total: number;
  label?: string;
  /** 显示百分比，默认 true */
  showPercent?: boolean;
  className?: string;
};

export function UsageMeter({
  used,
  total,
  label = '上下文',
  showPercent = true,
  className,
}: UsageMeterProps) {
  const ratio = total > 0 ? Math.min(1, Math.max(0, used / total)) : 0;
  const pct = Math.round(ratio * 100);
  const level = ratio >= 0.9 ? 'high' : ratio >= 0.7 ? 'mid' : 'low';
  const barClass = { low: 'bg-tint', mid: 'bg-orange', high: 'bg-red' }[level];
  const textClass = { low: 'text-tertiary-label', mid: 'text-orange', high: 'text-red' }[level];

  return (
    <div className={cn('flex items-center gap-2.5', className)}>
      <span className="shrink-0 text-xs text-tertiary-label">{label}</span>
      <div
        role="progressbar"
        aria-label={label}
        aria-valuenow={pct}
        aria-valuemin={0}
        aria-valuemax={100}
        className="h-1 w-full overflow-hidden rounded-full bg-fill"
      >
        <div
          className={cn('h-full rounded-full transition-[width] duration-500 ease-out', barClass)}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className={cn('shrink-0 font-mono text-xs tabular-nums', textClass)}>
        {formatTokens(used)} / {formatTokens(total)}
        {showPercent && ` · ${pct}%`}
      </span>
    </div>
  );
}

export type UsageBadgeProps = {
  /** 输入（提示词）tokens */
  prompt?: number;
  /** 输出（回复）tokens */
  completion?: number;
  className?: string;
};

const UpIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3"
    aria-hidden
  >
    <path d="M12 19V5M5 12l7-7 7 7" />
  </svg>
);

const DownIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3"
    aria-hidden
  >
    <path d="M12 5v14M19 12l-7 7-7-7" />
  </svg>
);

export function UsageBadge({ prompt, completion, className }: UsageBadgeProps) {
  const items: { key: string; icon: React.ReactNode; value: number }[] = [];
  if (prompt != null) items.push({ key: 'p', icon: <UpIcon />, value: prompt });
  if (completion != null) items.push({ key: 'c', icon: <DownIcon />, value: completion });
  if (items.length === 0) return null;

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-full bg-tertiary-fill px-2 py-0.5',
        'font-mono text-[11px] tabular-nums text-secondary-label',
        className,
      )}
    >
      {items.map((it) => (
        <span key={it.key} className="flex items-center gap-0.5">
          {it.icon}
          {formatTokens(it.value)}
        </span>
      ))}
    </span>
  );
}
