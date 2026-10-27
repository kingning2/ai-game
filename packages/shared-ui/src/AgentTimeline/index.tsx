import { cn } from '../cn';

/*
 * AgentTimeline —— Agent 运行时间线（多 Agent 组件的容器/底座）
 *
 * 规则（scrimui agent-console）：
 * - 事件**只追加、不覆盖**：重试是追加一条 linkedId 指向被接替事件的链接事件，
 *   绝不原地改写 —— 调试需要的历史就在这里
 * - 每条事件：图标点 + 左侧时间 + 标题 + 补充说明；左侧竖轨串联
 * - kind → 语义色：spawn=tint、tool=灰、handoff=blue、approval=orange、
 *   retry=orange、error=red、done=green、note=灰
 * - 纯数据驱动：events 由调用方 push 驱动（与 ToolExecution / WebSearch 同约定）
 */

export type AgentTimelineEventKind =
  | 'spawn'
  | 'tool'
  | 'handoff'
  | 'approval'
  | 'retry'
  | 'error'
  | 'done'
  | 'note';

export interface AgentTimelineEvent {
  /** 唯一 id；重试事件用 linkedId 指向被接替的事件 id */
  id: string;
  kind: AgentTimelineEventKind;
  /** 事件标题，如 '读取账单数据' */
  title: string;
  /** 补充说明，如工具名/耗时/错误信息 */
  detail?: string;
  /** 发生时间，如 '09:40:41' */
  time?: string;
  /** 重试/接替指向的原事件 id（追加链接事件，不覆盖历史） */
  linkedId?: string;
}

export interface AgentTimelineProps {
  events: AgentTimelineEvent[];
  title?: string;
  /** 事件点击回调（如跳转到对应 agent 卡片） */
  onOpenEvent?: (event: AgentTimelineEvent) => void;
  className?: string;
}

const kindIcon: Record<AgentTimelineEventKind, string> = {
  spawn: '▸',
  tool: '⚙',
  handoff: '⇄',
  approval: '◆',
  retry: '↻',
  error: '✕',
  done: '✓',
  note: '•',
};

const kindDotClass: Record<AgentTimelineEventKind, string> = {
  spawn: 'bg-tint/15 text-tint',
  tool: 'bg-tertiary-fill text-secondary-label',
  handoff: 'bg-blue/15 text-blue',
  approval: 'bg-orange/15 text-orange',
  retry: 'bg-orange/15 text-orange',
  error: 'bg-red/15 text-red',
  done: 'bg-green/15 text-green',
  note: 'bg-tertiary-fill text-tertiary-label',
};

const titleClass: Record<AgentTimelineEventKind, string> = {
  error: 'text-red',
  retry: 'text-orange',
  done: 'text-secondary-label',
  note: 'text-tertiary-label',
  spawn: 'text-label',
  tool: 'text-label',
  handoff: 'text-label',
  approval: 'text-label',
};

export function AgentTimeline({ events, title, onOpenEvent, className }: AgentTimelineProps) {
  const linkedIds = new Set(events.map((e) => e.linkedId).filter(Boolean) as string[]);

  return (
    <div
      className={cn(
        'overflow-hidden rounded-xl bg-secondary-grouped-background shadow-card',
        className,
      )}
    >
      {title && (
        <div className="px-4 pb-1 pt-3.5 text-[13px] font-semibold text-label">{title}</div>
      )}
      <ol className="px-4 pb-3 pt-2">
        {events.map((event, i) => {
          const isLast = i === events.length - 1;
          const superseded = linkedIds.has(event.id);
          const clickable = Boolean(onOpenEvent);
          const row = (
            <div
              className={cn(
                'flex items-start gap-2.5 transition-colors duration-150 ease-out',
                clickable && 'cursor-pointer hover:bg-tertiary-background/50',
                superseded && 'opacity-50',
              )}
            >
              {/* 左：时间 + 图标点 + 竖轨 */}
              <div className="flex w-14 shrink-0 flex-col items-end">
                <span className="text-[11px] tabular-nums text-tertiary-label">
                  {event.time ?? ''}
                </span>
              </div>
              <div className="flex flex-col items-center">
                <span
                  className={cn(
                    'flex size-5 shrink-0 items-center justify-center rounded-full text-[11px] leading-none',
                    kindDotClass[event.kind],
                  )}
                  aria-hidden
                >
                  {kindIcon[event.kind]}
                </span>
                {!isLast && <span className="w-px flex-1 bg-separator" aria-hidden />}
              </div>
              {/* 右：标题 + 说明 */}
              <div className={cn('min-w-0 flex-1', !isLast && 'pb-3')}>
                <div
                  className={cn(
                    'text-[13px] leading-snug',
                    titleClass[event.kind],
                    superseded && 'line-through',
                  )}
                >
                  {event.title}
                </div>
                {event.detail && (
                  <div className="mt-0.5 break-words text-xs leading-snug text-tertiary-label">
                    {event.detail}
                  </div>
                )}
                {event.linkedId && (
                  <div className="mt-0.5 text-[11px] text-orange">↻ 接替事件 {event.linkedId}</div>
                )}
              </div>
            </div>
          );
          return (
            <li
              key={event.id}
              onKeyDown={
                clickable
                  ? (e) => {
                      if (e.key === 'Enter') onOpenEvent?.(event);
                    }
                  : undefined
              }
            >
              {clickable ? (
                <button
                  type="button"
                  onClick={() => onOpenEvent?.(event)}
                  className="block w-full text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-tint"
                >
                  {row}
                </button>
              ) : (
                row
              )}
            </li>
          );
        })}
        {events.length === 0 && <li className="py-2 text-xs text-tertiary-label">暂无事件</li>}
      </ol>
    </div>
  );
}
