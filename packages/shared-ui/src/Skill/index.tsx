import type { ReactNode } from 'react';
import { cn } from '../cn';
import { Switch } from '../Switch';

/**
 * SkillCard —— 技能卡片，展示 Agent 可用的技能（skill）：
 * - 图标（默认 sparkle）+ 名称 + 描述 + 可选标签，纯展示、零业务逻辑
 * - 传入 onToggle 时右侧显示启用开关，是否启用由调用方状态驱动
 * - 描述两行截断，标签走令牌胶囊，全部 Apple HIG 语义色
 */

export type SkillCardProps = {
  /** 技能名，如 arkcli-billing / web-search */
  name: string;
  /** 技能描述（做什么、何时触发），两行截断 */
  description?: string;
  /** 分类标签，如 ['计费', '查询'] */
  tags?: string[];
  /** 自定义图标；不传用默认 sparkle */
  icon?: ReactNode;
  /** 是否启用（受控）；传入 onToggle 时展示开关 */
  enabled?: boolean;
  /** 切换启用回调；传入即显示开关 */
  onToggle?: (enabled: boolean) => void;
  className?: string;
};

const SparklesIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={1.8}
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-4"
    aria-hidden
  >
    <path d="M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9L12 3z" />
    <path d="M19 15l.9 2.1L22 18l-2.1.9L19 21l-.9-2.1L16 18l2.1-.9L19 15z" />
  </svg>
);

export function SkillCard({
  name,
  description,
  tags,
  icon,
  enabled = false,
  onToggle,
  className,
}: SkillCardProps) {
  return (
    <div
      className={cn(
        'flex items-start gap-3 rounded-xl bg-secondary-grouped-background p-3.5 shadow-card',
        className,
      )}
    >
      <div
        aria-hidden
        className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-tint/10 text-tint"
      >
        {icon ?? <SparklesIcon />}
      </div>
      <div className="min-w-0 flex-1">
        <p className="text-sm font-semibold text-label">{name}</p>
        {description ? (
          <p className="mt-0.5 line-clamp-2 text-xs leading-relaxed text-secondary-label">
            {description}
          </p>
        ) : null}
        {tags && tags.length > 0 ? (
          <div className="mt-2 flex flex-wrap gap-1.5">
            {tags.map((tag) => (
              <span
                key={tag}
                className="rounded-md bg-tertiary-fill px-1.5 py-0.5 text-xs text-secondary-label"
              >
                {tag}
              </span>
            ))}
          </div>
        ) : null}
      </div>
      {onToggle ? (
        <Switch
          checked={enabled}
          onCheckedChange={onToggle}
          aria-label={`启用 ${name}`}
          className="mt-0.5 shrink-0"
        />
      ) : null}
    </div>
  );
}
