import type { HTMLAttributes, ReactNode } from 'react';
import { cn } from '../cn';

export type ListProps = {
  className?: string;
  children?: ReactNode;
};

/** 分组列表容器 —— iOS 分组列表规格：divide-y 发丝线 + 分组单元格底色 */
export function List({ className, children }: ListProps) {
  return (
    <div
      className={cn(
        'w-full divide-y divide-separator overflow-hidden rounded-xl bg-secondary-grouped-background shadow-card',
        className,
      )}
    >
      {children}
    </div>
  );
}

export type ListItemProps = {
  /** 行首媒体区（头像 / 图标） */
  media?: ReactNode;
  title: ReactNode;
  subtitle?: ReactNode;
  /** 行尾（值文本 / 徽标 / 控件） */
  trailing?: ReactNode;
  onClick?: () => void;
  className?: string;
} & Omit<HTMLAttributes<HTMLElement>, 'onClick' | 'className'>;

/** 列表行：flex 一维布局（media | 内容弹性区 | trailing 固定区） */
export function ListItem({
  media,
  title,
  subtitle,
  trailing,
  onClick,
  className,
  ...rest
}: ListItemProps) {
  const Tag = onClick ? 'button' : 'div';
  return (
    <Tag
      onClick={onClick}
      className={cn(
        'flex w-full items-center gap-3 px-4 py-3 text-start text-sm',
        onClick &&
          'cursor-pointer transition duration-150 ease-out hover:bg-quaternary-fill active:scale-[0.98] focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-tint',
        className,
      )}
      {...rest}
    >
      {media}
      <span className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className="truncate text-label">{title}</span>
        {subtitle ? (
          <span className="truncate text-xs text-secondary-label">{subtitle}</span>
        ) : null}
      </span>
      {trailing ? (
        <span className="flex shrink-0 items-center gap-1 text-secondary-label">{trailing}</span>
      ) : null}
    </Tag>
  );
}
