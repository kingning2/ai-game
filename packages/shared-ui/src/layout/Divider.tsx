import { cn } from '../cn';

export type DividerProps = {
  vertical?: boolean;
  /** 列表内缩进分隔线（iOS 列表样式，跟随行内边距） */
  inset?: boolean;
  className?: string;
};

/** 发丝分隔线：半透明语义色（--separator），明暗主题下叠加在任何背景上都成立 */
export function Divider({ vertical = false, inset = false, className }: DividerProps) {
  return (
    <div
      role="separator"
      aria-orientation={vertical ? 'vertical' : 'horizontal'}
      className={cn(
        'bg-separator',
        vertical ? 'w-px self-stretch' : 'h-px w-full',
        inset && 'ms-4',
        className,
      )}
    />
  );
}
