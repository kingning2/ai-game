import { cn } from '../cn';

export type ProgressProps = {
  /** 0–100 */
  value?: number;
  className?: string;
};

const clamp = (n: number) => Math.min(100, Math.max(0, n));

/**
 * 进度条：宽度是运行时数据（style 注入），其余全部令牌；
 * 宽度变化带 500ms 强 ease-out 过渡（生长感）
 */
export function Progress({ value = 0, className }: ProgressProps) {
  const v = clamp(value);
  return (
    <div
      role="progressbar"
      aria-valuenow={v}
      aria-valuemin={0}
      aria-valuemax={100}
      className={cn('h-1.5 w-full overflow-hidden rounded-full bg-fill', className)}
    >
      <div
        className="h-full rounded-full bg-tint transition-[width] duration-500 ease-out"
        style={{ width: `${v}%` }}
      />
    </div>
  );
}
