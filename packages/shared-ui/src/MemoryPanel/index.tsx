import { useId, useState } from 'react';
import { cn } from '../cn';
import { Input } from '../Input';

/*
 * MemoryPanel —— Agent 记忆面板
 *
 * 数据模型对齐 pi 的 para-memory-files 记忆体系：记忆按类型分档
 * （fact 事实 / note 备注 / entity 实体 / plan 计划），带时间与置顶，支持检索。
 *
 * 规则（Apple HIG / emil-design-eng）：
 * - 检索框可选：传 query/onQueryChange 走受控；都不传则组件内过滤（纯展示逻辑，不碰业务状态）
 * - 类型徽标语义：fact=blue（可依赖的事实）、plan=orange（待执行意图）、
 *   entity=tint（结构化对象）、note=灰（弱信息）
 * - 置顶项固定在列表最前，始终不参与过滤排序
 * - 空态诚实：没有匹配就直说"没有找到"，不假装加载
 */

export type MemoryKind = 'fact' | 'note' | 'entity' | 'plan';

export interface MemoryEntry {
  /** 唯一 id */
  id: string;
  kind: MemoryKind;
  /** 记忆内容（一句话事实/备注/实体/计划） */
  content: string;
  /** 记录时间，如 '昨天 14:20' */
  time?: string;
  /** 置顶：固定在列表最前，不受过滤影响 */
  pinned?: boolean;
}

export interface MemoryPanelProps {
  entries: MemoryEntry[];
  title?: string;
  /** 受控搜索词；与 onQueryChange 搭配使用 */
  query?: string;
  onQueryChange?: (q: string) => void;
  /** 记忆条目点击回调 */
  onOpenEntry?: (entry: MemoryEntry) => void;
  /** 显示检索框（默认 true） */
  searchable?: boolean;
  className?: string;
}

const kindLabel: Record<MemoryKind, string> = {
  fact: '事实',
  note: '备注',
  entity: '实体',
  plan: '计划',
};

const kindBadgeClass: Record<MemoryKind, string> = {
  fact: 'bg-blue/15 text-blue',
  note: 'bg-tertiary-fill text-secondary-label',
  entity: 'bg-tint/10 text-tint',
  plan: 'bg-orange/15 text-orange',
};

const searchIcon = (
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
    <circle cx="11" cy="11" r="8" />
    <path d="m21 21-4.3-4.3" />
  </svg>
);

const pinIcon = (
  <svg
    viewBox="0 0 24 24"
    fill="currentColor"
    stroke="none"
    className="size-3 shrink-0 text-tertiary-label"
    aria-hidden
  >
    <path d="M14 3l7 7-4 1-3 6-2-2-5 5-1-1 5-5-2-2 6-3 1-4z" />
  </svg>
);

function sortEntries(entries: MemoryEntry[], q: string): MemoryEntry[] {
  const needle = q.trim().toLowerCase();
  const hit = (e: MemoryEntry) => !needle || e.content.toLowerCase().includes(needle);
  const pinned = entries.filter((e) => e.pinned && hit(e));
  const rest = entries.filter((e) => !e.pinned && hit(e));
  return [...pinned, ...rest];
}

export function MemoryPanel({
  entries,
  title = '记忆',
  query,
  onQueryChange,
  onOpenEntry,
  searchable = true,
  className,
}: MemoryPanelProps) {
  const autoId = useId();
  const [innerQuery, setInnerQuery] = useState('');
  const controlled = typeof query === 'string' && typeof onQueryChange === 'function';
  const q = controlled ? query : innerQuery;
  const setQ = (v: string) => {
    if (controlled) onQueryChange(v);
    else setInnerQuery(v);
  };

  const visible = sortEntries(entries, q);
  const filtered =
    q.trim().length > 0 &&
    visible.length <
      entries.filter((e) => e.pinned).length + entries.filter((e) => !e.pinned).length;

  return (
    <div
      className={cn(
        'overflow-hidden rounded-xl bg-secondary-grouped-background shadow-card',
        className,
      )}
    >
      <div className="flex items-center justify-between gap-3 px-4 pb-1 pt-3.5">
        <span className="text-[13px] font-semibold text-label">{title}</span>
        <span className="text-xs tabular-nums text-tertiary-label">{entries.length} 条</span>
      </div>

      {searchable && (
        <div className="px-4 pb-2 pt-1">
          <label htmlFor={autoId} className="sr-only">
            搜索记忆
          </label>
          <div className="relative">
            <span
              className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-tertiary-label"
              aria-hidden
            >
              {searchIcon}
            </span>
            <Input
              id={autoId}
              type="search"
              value={q}
              onChange={(e) => setQ(e.target.value)}
              placeholder="搜索记忆…"
              className="h-8 pl-8 text-xs"
            />
          </div>
        </div>
      )}

      <ul className="divide-y divide-separator/60 border-t border-separator/60">
        {visible.map((entry) => {
          const clickable = Boolean(onOpenEntry);
          const item = (
            <>
              <span
                className={cn(
                  'shrink-0 rounded-full px-2 py-0.5 text-[11px] leading-none',
                  kindBadgeClass[entry.kind],
                )}
              >
                {kindLabel[entry.kind]}
              </span>
              <span className="min-w-0 flex-1 break-words text-[13px] leading-snug text-label">
                {entry.content}
              </span>
              {entry.pinned && pinIcon}
              {entry.time && (
                <span className="shrink-0 text-[11px] tabular-nums text-tertiary-label">
                  {entry.time}
                </span>
              )}
            </>
          );
          return (
            <li key={entry.id} className="px-4">
              {clickable ? (
                <button
                  type="button"
                  onClick={() => onOpenEntry?.(entry)}
                  className="flex w-full items-center gap-2 py-2.5 text-left transition-colors duration-150 ease-out hover:bg-tertiary-background/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-tint"
                >
                  {item}
                </button>
              ) : (
                <div className="flex items-center gap-2 py-2.5">{item}</div>
              )}
            </li>
          );
        })}
        {visible.length === 0 && (
          <li className="px-4 py-4 text-center text-xs text-tertiary-label">
            {q.trim() ? '没有匹配的记忆' : '暂无记忆'}
          </li>
        )}
      </ul>
      {filtered && visible.length > 0 && (
        <div className="border-t border-separator/60 px-4 py-2 text-[11px] text-tertiary-label">
          已按“{q.trim()}”过滤
        </div>
      )}
    </div>
  );
}
