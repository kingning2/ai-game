import { cn } from '../cn';
import { Spinner } from '../Spinner';

/*
 * AgentTree —— 智能体委派树（orchestrator → subagent 层级）
 *
 * 规则（Claude Code agent-tree / LukeW task-list）：
 * - 缩进子任务天然表达并行与依赖：嵌套 ul + 缩进参考线（border-l），不画连线
 * - 每节点一行：状态点 + 名称 + 角色徽标 + 进行时活动文案，视觉焦点在"正在做什么"
 * - 状态语义与 AgentCard / TaskPlan 完全一致：running=Spinner、waiting=orange、
 *   done=green、failed=red、idle=描边空心
 * - 纯数据驱动：树结构完全由调用方维护，组件零内部状态、零时序假设
 */

export type AgentTreeNodeStatus = 'idle' | 'running' | 'waiting' | 'done' | 'failed';

export interface AgentTreeNode {
  /** 唯一标识 */
  id: string;
  /** 显示名 */
  name: string;
  /** 角色徽标，如 'orchestrator' / 'researcher' */
  role?: string;
  status: AgentTreeNodeStatus;
  /** 进行时活动文案 */
  activity?: string;
  /** 子委派（下一层 agent） */
  children?: AgentTreeNode[];
}

export interface AgentTreeProps {
  /** 树根（通常只有一个 orchestrator，也可传多根表示并行编排） */
  nodes: AgentTreeNode[];
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

function NodeDot({ status }: { status: AgentTreeNodeStatus }) {
  if (status === 'running') return <Spinner size="sm" className="size-4 text-tint" />;
  if (status === 'waiting')
    return <span className="size-2.5 rounded-full bg-orange animate-pulse" aria-hidden />;
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

const statusTextClass: Record<AgentTreeNodeStatus, string> = {
  idle: 'text-tertiary-label',
  running: 'text-label',
  waiting: 'text-orange',
  done: 'text-secondary-label',
  failed: 'text-red',
};

function TreeNodes({ nodes }: { nodes: AgentTreeNode[] }) {
  return (
    <ul className="min-w-0">
      {nodes.map((node) => (
        <li key={node.id} className="min-w-0">
          <div className="flex items-center gap-2 py-1.5">
            <span className="flex size-5 shrink-0 items-center justify-center">
              <NodeDot status={node.status} />
            </span>
            <span
              className={cn(
                'truncate text-[13px]',
                statusTextClass[node.status],
                node.status === 'running' && 'font-medium',
              )}
            >
              {node.name}
            </span>
            {node.role === 'orchestrator' ? (
              <span className="shrink-0 rounded-full bg-tint/10 px-2 py-0.5 text-[11px] leading-none text-tint">
                {node.role}
              </span>
            ) : node.role ? (
              <span className="shrink-0 rounded-full bg-tertiary-fill px-2 py-0.5 text-[11px] leading-none text-secondary-label">
                {node.role}
              </span>
            ) : null}
            {node.activity && node.status === 'running' && (
              <span className="min-w-0 flex-1 truncate text-xs text-tertiary-label">
                {node.activity}
              </span>
            )}
          </div>
          {node.children && node.children.length > 0 && (
            <div className="ml-2.5 border-l border-separator pl-4">
              <TreeNodes nodes={node.children} />
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}

export function AgentTree({ nodes, className }: AgentTreeProps) {
  return (
    <div
      className={cn(
        'overflow-hidden rounded-xl bg-secondary-grouped-background px-4 py-2.5 shadow-card',
        className,
      )}
    >
      <TreeNodes nodes={nodes} />
    </div>
  );
}
