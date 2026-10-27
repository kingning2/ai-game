import { createContext, useCallback, useContext, useMemo, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { cn } from '../cn';

/*
 * Toast —— 全局轻通知（后台任务完成/失败）
 *
 * 规则（Apple HIG / emil-design-eng）：
 * - iOS 横幅式：顶部居中、材质模糊（bg-material + backdrop-blur）、点击即关
 * - 入场 slide-in-top（抽屉曲线），退场 fade-out 后卸载 —— 保留两段动画，退场不硬切
 * - 同屏最多 3 条，超出移除最旧；自动关闭默认 3.2s
 */

export type ToastKind = 'info' | 'success' | 'error';

export interface ToastOptions {
  title: string;
  description?: string;
  kind?: ToastKind;
  /** 自动关闭毫秒数，默认 3200 */
  duration?: number;
}

interface ToastItem extends Required<Pick<ToastOptions, 'title' | 'kind'>> {
  id: number;
  description?: string;
}

interface ToastContextValue {
  /** 弹一条通知 */
  toast: (options: ToastOptions) => void;
  /** 手动关闭一条 */
  dismiss: (id: number) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error('useToast 必须在 <ToastProvider> 内使用');
  return ctx;
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

function KindIcon({ kind }: { kind: ToastKind }) {
  if (kind === 'success')
    return (
      <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-green/15 text-green">
        <CheckIcon />
      </span>
    );
  if (kind === 'error')
    return (
      <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-red/15 text-red">
        <XIcon />
      </span>
    );
  return <span className="mt-1.5 size-2 shrink-0 rounded-full bg-tint" aria-hidden />;
}

const MAX_VISIBLE = 3;
const LEAVE_MS = 180; // 与 animate-fade-out (0.15s) 对齐

export function ToastProvider({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<ToastItem[]>([]);
  const [leaving, setLeaving] = useState<ReadonlySet<number>>(new Set());
  const seq = useRef(0);

  const dismiss = useCallback((id: number) => {
    setLeaving((s) => new Set(s).add(id));
    window.setTimeout(() => {
      setItems((s) => s.filter((i) => i.id !== id));
      setLeaving((s) => {
        const next = new Set(s);
        next.delete(id);
        return next;
      });
    }, LEAVE_MS);
  }, []);

  const toast = useCallback(
    ({ title, description, kind = 'info', duration = 3200 }: ToastOptions) => {
      const id = ++seq.current;
      setItems((s) => [...s.slice(-(MAX_VISIBLE - 1)), { id, title, description, kind }]);
      window.setTimeout(() => dismiss(id), duration);
    },
    [dismiss],
  );

  const value = useMemo<ToastContextValue>(() => ({ toast, dismiss }), [toast, dismiss]);

  return (
    <ToastContext.Provider value={value}>
      {children}
      <div
        aria-live="polite"
        className="pointer-events-none fixed inset-x-0 top-3 z-50 flex flex-col items-center gap-2 px-3"
      >
        {items.map((item) => (
          <div
            key={item.id}
            role="status"
            onClick={() => dismiss(item.id)}
            className={cn(
              'pointer-events-auto flex w-full max-w-sm cursor-pointer items-start gap-2.5 rounded-xl px-4 py-3 shadow-card',
              'bg-material backdrop-blur-xl transition-colors duration-150 ease-out hover:bg-material/70',
              leaving.has(item.id) ? 'animate-fade-out' : 'animate-slide-in-top',
            )}
          >
            <KindIcon kind={item.kind} />
            <div className="min-w-0 flex-1">
              <div className="text-[13px] font-semibold text-label">{item.title}</div>
              {item.description && (
                <div className="mt-0.5 break-words text-xs leading-relaxed text-secondary-label">
                  {item.description}
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}
