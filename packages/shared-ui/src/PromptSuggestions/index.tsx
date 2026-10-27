import { cn } from '../cn';

/*
 * PromptSuggestions —— 快捷提示词（空状态 / 输入区上方）
 *
 * 规则（Apple HIG / emil-design-eng）：
 * - 默认横向 wrap 胶囊；stacked 切列表式（带箭头，适合引导页）
 * - stagger 入场：animate-rise + 每项 40ms delay（封顶 200ms，prefers-reduced-motion 下自动清零）
 * - 纯展示：点击把 prompt 交回调用方（通常填入 Composer）
 */

export interface PromptSuggestionsProps {
  prompts: string[];
  onSelect: (prompt: string) => void;
  /** 标题，如 '试试这些'；不传不显示 */
  title?: string;
  /** 列表式（带箭头）替代胶囊；适合引导页大按钮 */
  stacked?: boolean;
  className?: string;
}

const ArrowIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3.5 text-tertiary-label"
    aria-hidden
  >
    <path d="M5 12h14" />
    <path d="m12 5 7 7-7 7" />
  </svg>
);

export function PromptSuggestions({
  prompts,
  onSelect,
  title,
  stacked = false,
  className,
}: PromptSuggestionsProps) {
  return (
    <div className={cn('flex flex-col', className)}>
      {title && <div className="pb-2 text-xs font-semibold text-secondary-label">{title}</div>}
      <div className={cn(stacked ? 'flex flex-col gap-2' : 'flex flex-wrap gap-2')}>
        {prompts.map((prompt, i) =>
          stacked ? (
            <button
              key={prompt}
              type="button"
              onClick={() => onSelect(prompt)}
              style={{ animationDelay: `${Math.min(i * 40, 200)}ms` }}
              className="animate-rise flex w-full items-center justify-between gap-3 rounded-xl bg-secondary-grouped-background px-4 py-3 text-left text-sm text-label shadow-card transition-colors duration-150 ease-out hover:bg-tertiary-fill"
            >
              <span className="min-w-0 flex-1 truncate">{prompt}</span>
              <ArrowIcon />
            </button>
          ) : (
            <button
              key={prompt}
              type="button"
              onClick={() => onSelect(prompt)}
              style={{ animationDelay: `${Math.min(i * 40, 200)}ms` }}
              className="animate-rise rounded-full bg-secondary-grouped-background px-3.5 py-2 text-[13px] text-label shadow-card transition-colors duration-150 ease-out hover:bg-tertiary-fill"
            >
              {prompt}
            </button>
          ),
        )}
      </div>
    </div>
  );
}
