import { cn } from '../cn';

const sizeClass = {
  sm: 'size-4',
  md: 'size-5',
  lg: 'size-6',
} as const;

export type SpinnerProps = {
  size?: keyof typeof sizeClass;
  className?: string;
};

/** 加载指示器：颜色跟随文字色（默认次要文字），旋转用内置 animate-spin */
export function Spinner({ size = 'md', className }: SpinnerProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      role="status"
      aria-label="加载中"
      className={cn('animate-spin text-secondary-label', sizeClass[size], className)}
    >
      <circle cx="12" cy="12" r="9" stroke="currentColor" strokeOpacity="0.2" strokeWidth="3" />
      <path d="M21 12a9 9 0 0 0-9-9" stroke="currentColor" strokeWidth="3" strokeLinecap="round" />
    </svg>
  );
}
