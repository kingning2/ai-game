import { cn } from '../cn';
import { Spinner } from '../Spinner';
import { Avatar } from '../Avatar';

/*
 * AgentCard —— 智能体状态卡（多 Agent 协作的最小单元）
 *
 * 规则（scrimui agent-console / LukeW agent-management）：
 * - "谁在干什么"一眼可读：一卡一 agent，状态点 + 进行时活动文案 + 成本三件套（token/耗时/费用）
 * - 成本必须在卡上可见：只显示 fleet 总成本会让失控 agent 直到账单到来才被发现
 * - 失败的子任务是一张卡，不是一次宕机：失败态提供 per-agent Rerun（调用方驱动）
 * - 状态完全由调用方驱动：组件零内部状态、零时序假设（与 ToolExecution / TaskPlan 同约定）
 * - 等待审批/用户输入用 orange（Apple 语义"警告"），与 ApprovalCard 呼应
 */

export type AgentStatus = 'idle' | 'running' | 'waiting' | 'done' | 'failed';

export interface AgentCardData {
  /** 唯一标识 */
  id: string;
  /** 显示名，如 'Researcher' */
  name: string;
  /** 角色徽标文字，如 'orchestrator' / 'researcher'；orchestrator 会用蓝色徽标 */
  role?: string;
  /** 模型徽标文字，如 'doubao-seed-1.6' */
  model?: string;
  /** 进行时活动文案，如 '正在读取账单数据' */
  activity?: string;
  status: AgentStatus;
  /** 累计 token */
  tokens?: number;
  /** 累计成本（美元） */
  costUsd?: number;
  /** 运行耗时 */
  durationMs?: number;
}

export interface AgentCardProps {
  agent: AgentCardData;
  /** 选中态（配合 AgentTeamBoard：选中卡片切换右侧时间线，永不换页） */
  selected?: boolean;
  onSelect?: () => void;
  /** 失败后的重跑回调；不传则失败卡不显示 Rerun 按钮 */
  onRerun?: () => void;
  className?: string;
}

const CheckIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.6"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3"
    aria-hidden
  >
    <path d="m5 12 5 5 9-10" />
  </svg>
);

const XIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.6"
    strokeLinecap="round"
    className="size-3"
    aria-hidden
  >
    <path d="M18 6 6 18M6 6l12 12" />
  </svg>
);

/** 状态点：running 用 Spinner，waiting 橙色呼吸，done 绿勾，failed 红叉，idle 空心 */
function StatusDot({ status }: { status: AgentStatus }) {
  if (status === 'running') return <Spinner size="sm" className="size-4 text-tint" />;
  if (status === 'waiting')
    return (
      <span
        className="size-2.5 rounded-full bg-orange shadow-[0_0_0_0_rgba(255,149,0,0.4)] animate-pulse"
        aria-label="等待中"
      />
    );
  if (status === 'done')
    return (
      <span className="flex size-4 items-center justify-center rounded-full bg-green/15 text-green">
        <CheckIcon />
      </span>
    );
  if (status === 'failed')
    return (
      <span className="flex size-4 items-center justify-center rounded-full bg-red/15 text-red">
        <XIcon />
      </span>
    );
  return <span className="size-2.5 rounded-full border-2 border-tertiary-fill" aria-hidden />;
}

const statusLabel: Record<AgentStatus, string> = {
  idle: '空闲',
  running: '运行中',
  waiting: '等待输入',
  done: '已完成',
  failed: '失败',
};

const statusTextClass: Record<AgentStatus, string> = {
  idle: 'text-tertiary-label',
  running: 'text-tint',
  waiting: 'text-orange',
  done: 'text-secondary-label',
  failed: 'text-red',
};

function formatCost(usd: number) {
  if (usd < 0.01) return `$${usd.toFixed(4)}`;
  return `$${usd.toFixed(2)}`;
}

function formatDuration(ms: number) {
  if (ms < 1000) return `${Math.round(ms)}ms`;
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)}s`;
  const m = Math.floor(ms / 60_000);
  const s = Math.round((ms % 60_000) / 1000);
  return `${m}m ${s}s`;
}

function formatTokens(n: number) {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

export function AgentCard({ agent, selected, onSelect, onRerun, className }: AgentCardProps) {
  const { name, role, model, activity, status, tokens, costUsd, durationMs } = agent;
  const isOrchestrator = role === 'orchestrator';
  const initials = name.slice(0, 1).toUpperCase();
  const interactive = Boolean(onSelect);

  const body = (
    <>
      <div className="flex items-start gap-2.5">
        <Avatar size="sm" fallback={initials} aria-hidden />
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-1.5">
            <span className="truncate text-[13px] font-semibold text-label">{name}</span>
            {role && (
              <span
                className={cn(
                  'rounded-full px-2 py-0.5 text-[11px] leading-none',
                  isOrchestrator ? 'bg-tint/10 text-tint' : 'bg-tertiary-fill text-secondary-label',
                )}
              >
                {role}
              </span>
            )}
            {model && (
              <span className="max-w-[9rem] truncate rounded-full bg-tertiary-fill px-2 py-0.5 font-mono text-[11px] leading-none text-tertiary-label">
                {model}
              </span>
            )}
          </div>
          <div className="mt-1 flex items-center gap-1.5 text-xs">
            <StatusDot status={status} />
            <span
              className={cn(
                'truncate',
                status === 'running' && activity ? 'text-label' : statusTextClass[status],
              )}
            >
              {status === 'running' && activity ? activity : statusLabel[status]}
            </span>
          </div>
        </div>
      </div>

      <div className="mt-2.5 flex items-center gap-2.5 border-t border-separator/60 pt-2 text-[11px] tabular-nums text-tertiary-label">
        {typeof tokens === 'number' && <span title="累计 token">{formatTokens(tokens)} tok</span>}
        {typeof durationMs === 'number' && (
          <span title="运行耗时">{formatDuration(durationMs)}</span>
        )}
        {typeof costUsd === 'number' && (
          <span className={cn('font-medium', costUsd >= 0.5 && 'text-orange')} title="累计成本">
            {formatCost(costUsd)}
          </span>
        )}
        {status === 'failed' && onRerun && (
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onRerun();
            }}
            className="ml-auto rounded-md px-1.5 py-0.5 text-[11px] font-medium text-tint transition-colors duration-150 ease-out hover:bg-tint/10"
          >
            重跑
          </button>
        )}
      </div>
    </>
  );

  const shell = cn(
    'rounded-xl bg-secondary-grouped-background p-3 shadow-card transition-all duration-200 ease-out',
    interactive && 'cursor-pointer hover:shadow-popover active:scale-[0.98]',
    selected && 'ring-2 ring-tint',
    className,
  );

  if (!interactive) {
    return <div className={shell}>{body}</div>;
  }
  const handleSelect = onSelect;
  return (
    <div
      role="button"
      tabIndex={0}
      onClick={handleSelect}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          handleSelect?.();
        }
      }}
      className={cn(
        shell,
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-tint',
      )}
      aria-pressed={selected}
    >
      {body}
    </div>
  );
}
