import { useEffect, useRef } from 'react';
import { cn } from '../cn';

/**
 * 图片预览（Lightbox）：原生 <dialog> 模态 + 居中大图。
 * - showModal() 自动获得：Esc 关闭（cancel/close 事件）、焦点圈定、::backdrop 顶层
 * - body 滚动锁在 CSS：html:has(dialog:modal) { overflow: hidden }（styles/components.css）
 * - 进入动画用 @starting-style + allow-discrete（Baseline 2024-08）：遮罩淡入 + 图片放大，零 keyframes
 * - 点击遮罩 / Esc / 关闭按钮关闭
 */

export type LightboxProps = {
  src: string;
  alt?: string;
  /** 图片下方说明文字（如文件名） */
  caption?: string;
  onClose: () => void;
  className?: string;
};

const CloseIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-4"
    aria-hidden
  >
    <path d="M18 6L6 18M6 6l12 12" />
  </svg>
);

export function Lightbox({ src, alt, caption, onClose, className }: LightboxProps) {
  const ref = useRef<HTMLDialogElement>(null);

  // 挂载即模态打开；Esc 由平台处理并触发 onClose（dialog close 事件）
  useEffect(() => {
    ref.current?.showModal();
  }, []);

  return (
    <dialog
      ref={ref}
      onClose={onClose}
      onClick={onClose}
      aria-label={alt ?? '图片预览'}
      className={cn(
        'msgbox-lightbox m-auto bg-transparent p-6',
        'backdrop:bg-black/70 backdrop:backdrop-blur-sm',
        className,
      )}
    >
      <figure onClick={(e) => e.stopPropagation()} className="m-0">
        <img src={src} alt={alt ?? ''} className="h-[80vh] w-[min(92vw,1600px)] object-contain" />
        {caption && (
          <figcaption className="mt-3 text-center text-xs text-white/70">{caption}</figcaption>
        )}
      </figure>
      <button
        type="button"
        aria-label="关闭预览"
        onClick={(e) => {
          e.stopPropagation();
          onClose();
        }}
        className="absolute end-5 top-5 flex size-9 items-center justify-center rounded-full text-white bg-white/15 transition-colors hover:bg-white/25"
      >
        <CloseIcon />
      </button>
    </dialog>
  );
}
