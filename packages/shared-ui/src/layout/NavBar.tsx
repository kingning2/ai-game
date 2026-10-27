import type { HTMLAttributes, ReactNode } from 'react';
import { cn } from '../cn';

export type NavBarProps = Omit<HTMLAttributes<HTMLElement>, 'title'> & {
  /** 左侧（如返回箭头、Logo） */
  leading?: ReactNode;
  /** 中间标题（覆盖 DOM 语义的 string title）；未传时渲染 children */
  title?: ReactNode;
  /** 右侧动作区（如主题切换） */
  trailing?: ReactNode;
  /** 吸顶（默认开）；不吸顶时作为普通静态栏使用 */
  sticky?: boolean;
};

/**
 * 导航栏 —— iOS 毛玻璃材质的 web 形态：
 * - bg-material（半透明，令牌随明暗切换）+ backdrop-blur-xl + backdrop-saturate-150，
 *   滚动内容从栏底模糊透出（iOS 控制中心质感）
 * - prefers-reduced-transparency 时：--material 自动变实底，.material-blur 去模糊（令牌层降级）
 * - 底部发丝线分隔；内容宽度与 Container(size) 对齐由外部决定，内部只管排版
 */
export function NavBar({
  leading,
  title,
  trailing,
  sticky = true,
  className,
  children,
  ...props
}: NavBarProps) {
  return (
    <header
      className={cn(
        'material-blur border-b border-separator bg-material backdrop-blur-xl backdrop-saturate-150',
        sticky && 'sticky top-0 z-40',
        className,
      )}
      {...props}
    >
      <div className="mx-auto flex h-14 w-full max-w-xl items-center gap-3 px-4 sm:px-6">
        {leading}
        <div className="min-w-0 flex-1 truncate">{title ?? children}</div>
        {trailing}
      </div>
    </header>
  );
}
