import { cn } from '../cn';
import { Button } from '../Button';

/*
 * ApprovalCard —— Agent 敏感操作授权卡
 *
 * 规则（Apple HIG / emil-design-eng）：
 * - 人机协同关键组件：agent 要执行高风险操作（删文件/花钱/外发数据）时先请求授权
 * - 状态由调用方驱动：pending 显示按钮组，approved/denied 显示结果行 —— 与 ToolExecution 同约定
 * - 警示色用 orange（Apple 语义里的"警告"），按钮遵循 Button 变体规范：
 *   允许=solid 主操作、总是允许=tinted 次确认、拒绝=plain 覆盖 destructive 文字色
 */

export interface ApprovalRequest {
  /** 操作标题，如 '删除文件 src/legacy.ts' */
  title: string;
  /** 操作详情/风险说明（可选） */
  description?: string;
  /** 操作类型徽标，如 'bash' / 'file' / 'network' */
  kind?: string;
}

export interface ApprovalCardProps {
  request: ApprovalRequest;
  /** 授权状态：pending 显示按钮组，approved/denied 显示结果行，默认 'pending' */
  status?: 'pending' | 'approved' | 'denied';
  /** 允许一次 */
  onApprove?: () => void;
  /** 总是允许 */
  onAlwaysAllow?: () => void;
  /** 拒绝 */
  onDeny?: () => void;
  /** 已批准/拒绝后的补充说明，如 '本次会话内不再询问' */
  note?: string;
  className?: string;
}

const ShieldIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-4"
    aria-hidden
  >
    <path d="M12 3 5 6v5c0 4.4 3 8.4 7 10 4-1.6 7-5.6 7-10V6l-7-3Z" />
    <path d="M12 9v4" />
    <path d="M12 16h.01" />
  </svg>
);

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

export function ApprovalCard({
  request,
  status = 'pending',
  onApprove,
  onAlwaysAllow,
  onDeny,
  note,
  className,
}: ApprovalCardProps) {
  const { title, description, kind } = request;

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
            status === 'denied' ? 'bg-red/15 text-red' : 'bg-orange/15 text-orange',
          )}
        >
          <ShieldIcon />
        </span>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-[13px] font-semibold text-label">
              {status === 'pending' ? '需要你的批准' : status === 'approved' ? '已批准' : '已拒绝'}
            </span>
            {kind && (
              <span className="rounded-full bg-tertiary-fill px-2 py-0.5 text-[11px] text-secondary-label">
                {kind}
              </span>
            )}
          </div>
          <div className="mt-0.5 break-words text-sm text-secondary-label">{title}</div>
          {description && (
            <div className="mt-1 text-xs leading-relaxed text-tertiary-label">{description}</div>
          )}
          {status !== 'pending' && note && (
            <div className="mt-1 text-xs text-tertiary-label">{note}</div>
          )}
        </div>
        {status === 'approved' && (
          <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-green/15 text-green">
            <CheckIcon />
          </span>
        )}
        {status === 'denied' && (
          <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-red/15 text-red">
            <XIcon />
          </span>
        )}
      </div>

      {status === 'pending' && (
        <div className="flex items-center justify-end gap-2 border-t border-separator/60 bg-tertiary-background/50 px-4 py-2.5">
          <Button
            variant="plain"
            size="sm"
            onClick={onDeny}
            className="text-destructive hover:bg-destructive/10"
          >
            拒绝
          </Button>
          {onAlwaysAllow && (
            <Button variant="tinted" size="sm" onClick={onAlwaysAllow}>
              总是允许
            </Button>
          )}
          <Button variant="solid" size="sm" onClick={onApprove}>
            允许
          </Button>
        </div>
      )}
    </div>
  );
}
