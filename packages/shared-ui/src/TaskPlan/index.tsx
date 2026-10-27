import { cn } from '../cn';
import { Spinner } from '../Spinner';

/*
 * TaskPlan —— Agent 任务计划/步骤清单
 *
 * 规则（Apple HIG / emil-design-eng）：
 * - 纯数据驱动：steps 状态完全由调用方维护，组件零内部状态、零时序假设（与 ToolExecution 同约定）
 * - 语义色沿用 ToolExecution 体系：done=green、failed=red、running=tint Spinner、pending=描边空心
 * - 完成步骤弱化为次要文字，进行中步骤高亮 —— 视觉焦点永远在"正在做什么"
 */

export type TaskStepStatus = 'pending' | 'running' | 'done' | 'failed';

export interface TaskStep {
  /** 步骤标题，如 '解析账单周期' */
  title: string;
  /** 补充说明（可选），如工具名或耗时 */
  detail?: string;
  status: TaskStepStatus;
}

export interface TaskPlanProps {
  /** 计划标题，默认 '任务计划' */
  title?: string;
  steps: TaskStep[];
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

const XIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.4"
    strokeLinecap="round"
    className="size-3"
    aria-hidden
  >
    <path d="M18 6 6 18M6 6l12 12" />
  </svg>
);

function StepIcon({ status }: { status: TaskStepStatus }) {
  if (status === 'running') return <Spinner size="sm" className="size-4 text-tint" />;
  if (status === 'done')
    return (
      <span className="flex size-5 items-center justify-center rounded-full bg-green/15 text-green">
        <CheckIcon />
      </span>
    );
  if (status === 'failed')
    return (
      <span className="flex size-5 items-center justify-center rounded-full bg-red/15 text-red">
        <XIcon />
      </span>
    );
  return <span className="size-3.5 rounded-full border-2 border-tertiary-fill" aria-hidden />;
}

const titleClass: Record<TaskStepStatus, string> = {
  pending: 'text-tertiary-label',
  running: 'font-medium text-label',
  done: 'text-secondary-label',
  failed: 'text-label',
};

export function TaskPlan({ title = '任务计划', steps, className }: TaskPlanProps) {
  const doneCount = steps.filter((s) => s.status === 'done').length;
  const running = steps.some((s) => s.status === 'running');

  return (
    <div
      className={cn(
        'overflow-hidden rounded-xl bg-secondary-grouped-background shadow-card',
        className,
      )}
    >
      <div className="flex items-center justify-between gap-3 px-4 pb-1 pt-3.5">
        <span className="text-[13px] font-semibold text-label">{title}</span>
        <span className="text-xs tabular-nums text-tertiary-label">
          {doneCount}/{steps.length} 步
        </span>
      </div>

      <ul className="divide-y divide-separator/60 px-4 pb-2.5 pt-1">
        {steps.map((step, i) => (
          <li
            key={i}
            className={cn(
              'flex items-start gap-3 py-2.5 transition-colors duration-200 ease-out',
              step.status === 'running' && 'text-label',
            )}
          >
            <span className="flex size-5 shrink-0 items-center justify-center">
              <StepIcon status={step.status} />
            </span>
            <div className="min-w-0 flex-1">
              <div className={cn('text-sm leading-snug', titleClass[step.status])}>
                {step.title}
              </div>
              {step.detail && (
                <div className="mt-0.5 truncate text-xs text-tertiary-label">{step.detail}</div>
              )}
            </div>
          </li>
        ))}
      </ul>

      {running && (
        <div className="border-t border-separator/60 px-4 py-2.5 text-xs text-secondary-label">
          正在执行当前步骤…
        </div>
      )}
    </div>
  );
}
