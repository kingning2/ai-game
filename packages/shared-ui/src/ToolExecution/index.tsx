import { useState, type ReactNode } from 'react';
import { cn } from '../cn';
import { Spinner } from '../Spinner';

export type ToolExecutionStatus = 'pending' | 'running' | 'success' | 'error';

export type ToolExecutionData = {
  id: string;
  /** 工具名，如 web_search / bash */
  name: string;
  /** 任意可序列化入参 */
  args?: unknown;
  status: ToolExecutionStatus;
  /** 成功输出（文本） */
  result?: string;
  /** 失败信息 */
  error?: string;
  /** 耗时，结束后由调用方回填 */
  durationMs?: number;
};

const checkIcon = (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={3}
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3"
    aria-hidden
  >
    <path d="M5 13l4 4L19 7" />
  </svg>
);

const xIcon = (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={3}
    strokeLinecap="round"
    className="size-3"
    aria-hidden
  >
    <path d="M6 6l12 12M18 6L6 18" />
  </svg>
);

const chevronIcon = (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={2.5}
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-4"
    aria-hidden
  >
    <path d="M9 6l6 6-6 6" />
  </svg>
);

const statusIcon: Record<ToolExecutionStatus, ReactNode> = {
  pending: <span className="size-2 rounded-full bg-quaternary-label" />,
  running: <Spinner size="sm" />,
  success: (
    <span className="flex size-5 items-center justify-center rounded-full bg-green/15 text-green">
      {checkIcon}
    </span>
  ),
  error: (
    <span className="flex size-5 items-center justify-center rounded-full bg-red/15 text-red">
      {xIcon}
    </span>
  ),
};

const formatDuration = (ms: number) => (ms < 1000 ? `${ms}ms` : `${(ms / 1000).toFixed(1)}s`);

const formatArgs = (args: unknown) => {
  try {
    return JSON.stringify(args, null, 2) ?? '';
  } catch {
    return String(args);
  }
};

const outputClass =
  'overflow-x-auto rounded-lg bg-tertiary-fill p-2.5 font-mono text-xs whitespace-pre-wrap break-words';

export type ToolExecutionProps = {
  execution: ToolExecutionData;
  defaultOpen?: boolean;
  /** 失败时显示重试按钮 */
  onRetry?: () => void;
  className?: string;
};

/**
 * 工具执行卡片 —— agent 工具调用的可视化：
 * - 纯状态驱动：status 字段决定图标（待执行圆点 / Spinner / 绿勾 / 红叉），组件不做时序假设
 * - 头部整行可点披露参数与输出；披露动画 grid-rows 0fr→1fr（唯一合理的无魔法数字方案）
 * - 箭头旋转 200ms ease-out；错误态 tint 红字输出块 + 可选重试
 */
export function ToolExecution({
  execution,
  defaultOpen = false,
  onRetry,
  className,
}: ToolExecutionProps) {
  const [open, setOpen] = useState(defaultOpen);
  const { name, args, status, result, error, durationMs } = execution;
  const settled = status === 'success' || status === 'error';
  const hasBody = args !== undefined || Boolean(result) || Boolean(error);

  return (
    <div className={cn('rounded-xl bg-secondary-grouped-background shadow-card', className)}>
      <button
        type="button"
        onClick={() => hasBody && setOpen((o) => !o)}
        disabled={!hasBody}
        aria-expanded={hasBody ? open : undefined}
        className={cn(
          'flex w-full items-center gap-2.5 rounded-xl p-3 text-start text-sm',
          'transition duration-150 ease-out active:scale-[0.98]',
          hasBody ? 'hover:bg-quaternary-fill' : 'cursor-default',
        )}
      >
        <span className="flex shrink-0 items-center justify-center">{statusIcon[status]}</span>
        <span className="truncate font-mono text-sm text-label">{name}</span>
        <span className="ml-auto flex shrink-0 items-center gap-2">
          {settled && durationMs !== undefined ? (
            <span className="text-xs tabular-nums text-tertiary-label">
              {formatDuration(durationMs)}
            </span>
          ) : null}
          {hasBody ? (
            <span
              className={cn(
                'text-tertiary-label transition-transform duration-200 ease-out',
                open && 'rotate-90',
              )}
            >
              {chevronIcon}
            </span>
          ) : null}
        </span>
      </button>

      {hasBody ? (
        <div
          className={cn(
            'grid transition-[grid-template-rows] duration-300 ease-out',
            open ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]',
          )}
        >
          <div className="overflow-hidden">
            <div className="flex flex-col items-start gap-2 px-3 pb-3">
              {args !== undefined ? <pre className={outputClass}>{formatArgs(args)}</pre> : null}
              {error ? <pre className={cn(outputClass, 'bg-red/10 text-red')}>{error}</pre> : null}
              {result ? (
                <pre className={cn(outputClass, 'max-h-64 overflow-y-auto')}>{result}</pre>
              ) : null}
              {status === 'error' && onRetry ? (
                <button
                  type="button"
                  onClick={onRetry}
                  className="rounded-md bg-tint/10 px-3 py-1 text-xs font-medium text-tint transition duration-150 ease-out hover:bg-tint/15 active:scale-[0.97]"
                >
                  重试
                </button>
              ) : null}
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
