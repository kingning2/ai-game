import { useState } from 'react';
import { cn } from '../cn';
import { Button } from '../Button';

/*
 * ConversationList —— 多会话列表（Agent 系统入口）
 *
 * 规则（Apple HIG / emil-design-eng）：
 * - activeId 受控高亮；选择/重命名/删除都只发回调，数据归调用方
 * - 传 date 自动分「今天 / 最近 7 天 / 更早」三组；不传则平铺
 * - 重命名/删除按钮 hover 才出现（纯 CSS opacity，无 JS 开销）；重命名是行内 input，
 *   Enter/失焦提交、Esc 取消 —— 唯一允许的内部状态是 UI 态（editingId/draft）
 */

export interface Conversation {
  id: string;
  /** 会话标题 */
  title: string;
  /** 最近一条消息预览，一行截断 */
  preview?: string;
  /** 展示时间，如 '14:32' / '昨天' */
  time?: string;
  /** 会话时间，用于自动分组（今天/最近 7 天/更早）；不传归入"更早" */
  date?: Date;
}

export interface ConversationListProps {
  conversations: Conversation[];
  /** 当前激活会话 id（受控） */
  activeId?: string;
  onSelect?: (id: string) => void;
  onRename?: (id: string, title: string) => void;
  onDelete?: (id: string) => void;
  /** 顶部"新建会话"按钮（传入即显示） */
  onNew?: () => void;
  className?: string;
}

const GROUP_LABELS = ['今天', '最近 7 天', '更早'] as const;
const DAY_MS = 86_400_000;

function groupIndex(date?: Date): 0 | 1 | 2 {
  if (!date) return 2;
  const now = new Date();
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  if (date.getTime() >= startOfToday) return 0;
  if (date.getTime() >= startOfToday - 7 * DAY_MS) return 1;
  return 2;
}

const PlusIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    className="size-4"
    aria-hidden
  >
    <path d="M12 5v14M5 12h14" />
  </svg>
);

const PencilIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3.5"
    aria-hidden
  >
    <path d="M17 3a2.8 2.8 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
  </svg>
);

const TrashIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3.5"
    aria-hidden
  >
    <path d="M3 6h18M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2m3 0v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
  </svg>
);

export function ConversationList({
  conversations,
  activeId,
  onSelect,
  onRename,
  onDelete,
  onNew,
  className,
}: ConversationListProps) {
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draft, setDraft] = useState('');

  const startEdit = (c: Conversation) => {
    setEditingId(c.id);
    setDraft(c.title);
  };
  const commitEdit = () => {
    if (editingId && draft.trim() && onRename) onRename(editingId, draft.trim());
    setEditingId(null);
  };
  const cancelEdit = () => setEditingId(null);

  return (
    <div className={cn('flex flex-col gap-1', className)}>
      {onNew && (
        <Button variant="gray" size="sm" onClick={onNew} className="mb-1 w-full justify-start">
          <PlusIcon />
          新建会话
        </Button>
      )}

      {GROUP_LABELS.map((label, gi) => {
        const items = conversations.filter((c) => groupIndex(c.date) === gi);
        if (items.length === 0) return null;
        return (
          <div key={label} className="flex flex-col gap-0.5">
            <div className="px-2.5 pb-1 pt-2.5 text-[11px] font-semibold uppercase tracking-wide text-tertiary-label">
              {label}
            </div>
            {items.map((c) => {
              const active = c.id === activeId;
              const editing = c.id === editingId;
              return (
                <div
                  key={c.id}
                  role="button"
                  tabIndex={0}
                  onClick={() => !editing && onSelect?.(c.id)}
                  onKeyDown={(e) => {
                    if (!editing && (e.key === 'Enter' || e.key === ' ')) {
                      e.preventDefault();
                      onSelect?.(c.id);
                    }
                  }}
                  aria-current={active ? 'true' : undefined}
                  className={cn(
                    'group relative cursor-pointer rounded-lg px-2.5 py-2 transition-colors duration-150 ease-out outline-none',
                    active ? 'bg-tint/10' : 'hover:bg-tertiary-fill/50',
                    'focus-visible:ring-2 focus-visible:ring-tint/40',
                  )}
                >
                  {editing ? (
                    <input
                      autoFocus
                      value={draft}
                      onChange={(e) => setDraft(e.target.value)}
                      onBlur={commitEdit}
                      onClick={(e) => e.stopPropagation()}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') commitEdit();
                        if (e.key === 'Escape') cancelEdit();
                      }}
                      aria-label="重命名会话"
                      className="w-full rounded-md bg-transparent text-sm text-label outline-none ring-1 ring-tint/50"
                    />
                  ) : (
                    <div className="flex items-baseline justify-between gap-2">
                      <span
                        className={cn(
                          'truncate text-sm',
                          active ? 'font-medium text-label' : 'text-label',
                        )}
                      >
                        {c.title}
                      </span>
                      {c.time && (
                        <span className="shrink-0 text-[11px] text-tertiary-label">{c.time}</span>
                      )}
                    </div>
                  )}

                  {!editing && (
                    <>
                      {c.preview && (
                        <div className="mt-0.5 truncate pr-12 text-xs text-tertiary-label">
                          {c.preview}
                        </div>
                      )}
                      <div className="absolute right-1.5 top-2 flex items-center gap-0.5 opacity-0 transition-opacity duration-150 ease-out focus-within:opacity-100 group-hover:opacity-100">
                        {onRename && (
                          <button
                            type="button"
                            aria-label={`重命名 ${c.title}`}
                            onClick={(e) => {
                              e.stopPropagation();
                              startEdit(c);
                            }}
                            className="flex size-6 items-center justify-center rounded-md text-tertiary-label transition-colors duration-150 ease-out hover:bg-tertiary-fill hover:text-label"
                          >
                            <PencilIcon />
                          </button>
                        )}
                        {onDelete && (
                          <button
                            type="button"
                            aria-label={`删除 ${c.title}`}
                            onClick={(e) => {
                              e.stopPropagation();
                              onDelete(c.id);
                            }}
                            className="flex size-6 items-center justify-center rounded-md text-tertiary-label transition-colors duration-150 ease-out hover:bg-destructive/10 hover:text-destructive"
                          >
                            <TrashIcon />
                          </button>
                        )}
                      </div>
                    </>
                  )}
                </div>
              );
            })}
          </div>
        );
      })}
    </div>
  );
}
