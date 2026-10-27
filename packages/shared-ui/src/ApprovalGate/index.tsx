import { cn } from '../cn';
import { Spinner } from '../Spinner';

/*
 * ApprovalGate —— Agent 决策门（多 Agent 场景的结构化审批）
 *
 * 与 ApprovalCard 的分工：ApprovalCard 是单次敏感操作的 允许/拒绝 授权；
 * ApprovalGate 是 agent 在关键节点请求的**结构化决策**（2-4 个选项，如方案取舍/继续与否），
 * 对齐 pi 的 plan_mode_question 模型（选项 + 说明，含自由输入）。
 *
 * 规则（scrimui agent-console）：
 * - 内联在发起决策的 agent 自己的日志流里，决策时日志就在旁边 —— 不漂浮成全局 modal
 * - pending 显示选项按钮组；decided 后收拢为结果行（选中项 + 决策说明）
 * - 状态完全由调用方驱动：组件零内部状态、零时序假设（与 ToolExecution 同约定）
 */

export interface ApprovalOption {
  /** 选项 id，onDecide 回传 */
  id: string;
  /** 选项标签，如 '拆分为三个子任务' */
  label: string;
  /** 选项说明（可选） */
  description?: string;
}

export interface ApprovalGateProps {
  /** 决策请求标题，如 'Researcher 请求决策：结果如何落地' */
  title: string;
  /** 决策背景说明（可选） */
  description?: string;
  /** 结构化选项（2-4 个） */
  options: ApprovalOption[];
  /** 决策状态：pending 显示选项组，decided 收拢为结果行 */
  status?: 'pending' | 'decided';
  /** decided 时被选中的选项 id */
  decidedOptionId?: string;
  /** decided 后的补充说明，如 '按方案 A 继续，完成后汇报' */
  note?: string;
  onDecide?: (optionId: string) => void;
  className?: string;
}

const CheckIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.4"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3"
    aria-hidden
  >
    <path d="m5 12 5 5 9-10" />
  </svg>
);

/** 取文字首字符做头像缩写（中英通吃） */
function initialOf(text: string) {
  const m = text.match(/[\p{L}\p{N}]/u);
  return m ? m[0].toUpperCase() : '•';
}

export function ApprovalGate({
  title,
  description,
  options,
  status = 'pending',
  decidedOptionId,
  note,
  onDecide,
  className,
}: ApprovalGateProps) {
  const decided = status === 'decided';
  const chosen = options.find((o) => o.id === decidedOptionId);

  return (
    <div
      className={cn(
        'overflow-hidden rounded-xl bg-secondary-grouped-background shadow-card',
        className,
      )}
    >
      <div className="flex items-start gap-3 px-4 pb-3 pt-3.5">
        <span
          className={cn(
            'flex size-8 shrink-0 items-center justify-center rounded-lg',
            decided ? 'bg-green/15 text-green' : 'bg-orange/15 text-orange',
          )}
        >
          {decided ? <CheckIcon /> : <Spinner size="sm" className="size-4 text-orange" />}
        </span>
        <div className="min-w-0 flex-1">
          <span
            className={cn(
              'text-[13px] font-semibold',
              decided ? 'text-secondary-label' : 'text-label',
            )}
          >
            {title}
          </span>
          {description && (
            <div className="mt-0.5 text-xs leading-relaxed text-tertiary-label">{description}</div>
          )}
        </div>
      </div>

      {!decided && (
        <div className="space-y-1.5 border-t border-separator/60 bg-tertiary-background/50 px-4 py-3">
          {options.map((opt) => (
            <button
              key={opt.id}
              type="button"
              onClick={() => onDecide?.(opt.id)}
              className="flex w-full items-start gap-2.5 rounded-lg bg-secondary-grouped-background px-3 py-2 text-left shadow-card transition-all duration-150 ease-out hover:shadow-popover active:scale-[0.99] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-tint"
            >
              <span
                className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border-2 border-tertiary-fill"
                aria-hidden
              />
              <span className="min-w-0 flex-1">
                <span className="block text-[13px] font-medium text-label">{opt.label}</span>
                {opt.description && (
                  <span className="mt-0.5 block text-xs leading-snug text-tertiary-label">
                    {opt.description}
                  </span>
                )}
              </span>
            </button>
          ))}
        </div>
      )}

      {decided && chosen && (
        <div className="border-t border-separator/60 bg-tertiary-background/50 px-4 py-2.5">
          <div className="flex items-center gap-2 text-[13px] text-label">
            <span className="flex size-4 shrink-0 items-center justify-center rounded-full bg-green/15 text-green">
              <CheckIcon />
            </span>
            <span className="min-w-0 flex-1 truncate">{chosen.label}</span>
          </div>
          {note && <div className="mt-1 pl-6 text-xs text-tertiary-label">{note}</div>}
        </div>
      )}
      {/* decided 但找不到选项时兜底显示原选项 id，保持可审计 */}
      {decided && !chosen && decidedOptionId && (
        <div className="border-t border-separator/60 px-4 py-2.5 font-mono text-xs text-secondary-label">
          {initialOf(decidedOptionId)} {decidedOptionId}
        </div>
      )}
    </div>
  );
}
