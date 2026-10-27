import { cn } from '../cn';
import { AgentCard, type AgentCardData } from '../AgentCard';

/*
 * AgentTeamBoard —— 智能体团队面板（fleet roster）
 *
 * 规则（scrimui agent-console）：
 * - "谁在干什么"一屏可读：AgentCard 网格 + fleet 头部汇总（运行数/等待数/总成本）
 * - 待审批数在 fleet 头部汇总计数，但审批操作内联在对应 agent 的时间线里（ApprovalGate），
 *   头部按钮只负责跳转定位 —— 决策永远不漂浮在 modal 里
 * - 成本双显示：单卡显示 per-run 成本，这里汇总 fleet 总成本；总成本超高转 orange 警示
 */

export type { AgentCardData, AgentStatus } from '../AgentCard';

export interface AgentTeamBoardProps {
  agents: AgentCardData[];
  title?: string;
  /** 当前选中的 agent id */
  selectedId?: string;
  onSelectAgent?: (id: string) => void;
  onRerunAgent?: (id: string) => void;
  /** fleet 级待审批数；>0 时头部显示橙色徽标 */
  pendingApprovals?: number;
  /** 点击待审批徽标的回调（跳转到对应审批门） */
  onOpenApprovals?: () => void;
  className?: string;
}

function sumAgents(agents: AgentCardData[]) {
  let tokens = 0;
  let costUsd = 0;
  let running = 0;
  let waiting = 0;
  let failed = 0;
  for (const a of agents) {
    tokens += a.tokens ?? 0;
    costUsd += a.costUsd ?? 0;
    if (a.status === 'running') running += 1;
    if (a.status === 'waiting') waiting += 1;
    if (a.status === 'failed') failed += 1;
  }
  return { tokens, costUsd, running, waiting, failed };
}

export function AgentTeamBoard({
  agents,
  title = '智能体团队',
  selectedId,
  onSelectAgent,
  onRerunAgent,
  pendingApprovals = 0,
  onOpenApprovals,
  className,
}: AgentTeamBoardProps) {
  const { tokens, costUsd, running, waiting, failed } = sumAgents(agents);

  return (
    <div className={cn('rounded-xl bg-grouped-background p-3 shadow-card', className)}>
      {/* fleet 头部：运行数 / 等待数 / 待审批 / 总成本 */}
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5 px-1 pb-2.5 pt-0.5">
        <span className="text-[13px] font-semibold text-label">{title}</span>
        <span className="text-xs tabular-nums text-secondary-label">
          {running > 0 ? `${running} 运行中` : '无运行任务'}
          {waiting > 0 && <span className="text-orange"> · {waiting} 等待输入</span>}
          {failed > 0 && <span className="text-red"> · {failed} 失败</span>}
        </span>
        <span className="ml-auto flex items-center gap-2.5 text-xs tabular-nums text-tertiary-label">
          {tokens > 0 && (
            <span title="fleet 累计 token">
              {tokens >= 1000 ? `${(tokens / 1000).toFixed(1)}K` : tokens} tok
            </span>
          )}
          {costUsd > 0 && (
            <span
              className={cn('font-medium', costUsd >= 1 ? 'text-orange' : 'text-secondary-label')}
              title="fleet 累计成本"
            >
              ${costUsd.toFixed(2)}
            </span>
          )}
          {pendingApprovals > 0 && (
            <button
              type="button"
              onClick={onOpenApprovals}
              className="flex items-center gap-1 rounded-full bg-orange/15 px-2 py-0.5 text-[11px] font-medium text-orange transition-colors duration-150 ease-out hover:bg-orange/25"
            >
              <span className="size-1.5 rounded-full bg-orange" aria-hidden />
              {pendingApprovals} 待审批
            </button>
          )}
        </span>
      </div>

      {/* 卡片网格：flex-wrap 等宽卡片，选中卡切换时间线而非换页 */}
      <div className="flex flex-wrap gap-2">
        {agents.map((agent) => (
          <AgentCard
            key={agent.id}
            agent={agent}
            className="min-w-[13rem] flex-1 basis-[13rem]"
            selected={agent.id === selectedId}
            onSelect={onSelectAgent ? () => onSelectAgent(agent.id) : undefined}
            onRerun={onRerunAgent ? () => onRerunAgent(agent.id) : undefined}
          />
        ))}
      </div>
    </div>
  );
}
