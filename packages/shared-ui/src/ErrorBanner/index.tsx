import { cn } from '../cn';
import { Button } from '../Button';
import type { ApiErrorKind } from '@agent-packages/shared';

/*
 * ErrorBanner —— 失败横幅（接 @agent-packages/shared 的 ApiError 体系）
 *
 * 规则（Apple HIG / emil-design-eng）：
 * - kind 直接复用 shared 的 ApiErrorKind，默认文案按类别区分 —— 上层只管把 error 传下来
 * - 重试/关闭都是可选回调，传入即显示；组件不持有错误状态
 */

export interface ErrorBannerProps {
  /** 错误标题；不传按 kind 取默认文案 */
  title?: string;
  /** 错误详情（原始 message） */
  message?: string;
  /** 错误类别，决定默认文案；与 ApiError.kind 对齐 */
  kind?: ApiErrorKind;
  onRetry?: () => void;
  onDismiss?: () => void;
  className?: string;
}

const DEFAULT_TITLES: Record<ApiErrorKind, string> = {
  business: '请求被服务端拒绝',
  http: '服务暂时不可用',
  network: '网络连接失败',
  abort: '请求已取消',
  parse: '响应格式异常',
};

const AlertIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    className="size-4"
    aria-hidden
  >
    <circle cx="12" cy="12" r="9" />
    <path d="M12 8v4" />
    <path d="M12 16h.01" />
  </svg>
);

const CloseIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    className="size-3.5"
    aria-hidden
  >
    <path d="M18 6 6 18M6 6l12 12" />
  </svg>
);

export function ErrorBanner({
  title,
  message,
  kind,
  onRetry,
  onDismiss,
  className,
}: ErrorBannerProps) {
  return (
    <div
      role="alert"
      className={cn(
        'flex items-start gap-3 rounded-xl border border-destructive/20 bg-destructive/10 px-4 py-3',
        className,
      )}
    >
      <span className="mt-0.5 shrink-0 text-destructive">
        <AlertIcon />
      </span>
      <div className="min-w-0 flex-1">
        <div className="text-[13px] font-semibold text-label">
          {title ?? (kind ? DEFAULT_TITLES[kind] : '请求失败')}
        </div>
        {message && (
          <div className="mt-0.5 break-words text-xs leading-relaxed text-secondary-label">
            {message}
          </div>
        )}
      </div>
      <div className="flex shrink-0 items-center gap-1">
        {onRetry && (
          <Button
            variant="plain"
            size="sm"
            onClick={onRetry}
            className="h-7 text-destructive hover:bg-destructive/10"
          >
            重试
          </Button>
        )}
        {onDismiss && (
          <button
            type="button"
            aria-label="关闭"
            onClick={onDismiss}
            className="flex size-6 items-center justify-center rounded-md text-tertiary-label transition-colors duration-150 ease-out hover:bg-destructive/10 hover:text-label"
          >
            <CloseIcon />
          </button>
        )}
      </div>
    </div>
  );
}
