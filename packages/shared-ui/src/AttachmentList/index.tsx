import { useState } from 'react';
import { cn } from '../cn';
import { Lightbox } from '../Lightbox';

/**
 * 附件缩略图条：圆角缩略图 + hover 删除按钮 + 点击全屏预览（Lightbox）。
 * 通用展示组件（无内边距），由调用方控制外边距与数据管理。
 */

export type AttachmentItem = {
  id: string;
  /** 图片地址（本地用 URL.createObjectURL(file)，或远程 url） */
  url: string;
  name?: string;
};

export type AttachmentListProps = {
  items: AttachmentItem[];
  onRemove?: (id: string) => void;
  /** 缩略图边长（px），默认 64 */
  size?: number;
  /** 点击缩略图全屏预览，默认开启 */
  preview?: boolean;
  className?: string;
};

export function AttachmentList({
  items,
  onRemove,
  size = 64,
  preview = true,
  className,
}: AttachmentListProps) {
  const [previewItem, setPreviewItem] = useState<AttachmentItem | null>(null);
  if (items.length === 0) return null;

  return (
    <>
      <div className={cn('flex flex-wrap gap-2', className)}>
        {items.map((a) => (
          <div
            key={a.id}
            className="group relative shrink-0 overflow-hidden rounded-lg border border-separator"
            style={{ width: size, height: size }}
          >
            {preview ? (
              <button
                type="button"
                aria-label={`预览 ${a.name ?? '图片'}`}
                onClick={() => setPreviewItem(a)}
                className="size-full cursor-zoom-in"
              >
                <img src={a.url} alt={a.name ?? '附件图片'} className="size-full object-cover" />
              </button>
            ) : (
              <img src={a.url} alt={a.name ?? '附件图片'} className="size-full object-cover" />
            )}
            {onRemove && (
              <button
                type="button"
                aria-label={`移除 ${a.name ?? '图片'}`}
                onClick={() => onRemove(a.id)}
                className={cn(
                  'absolute end-1 top-1 flex size-5 items-center justify-center rounded-full bg-black/55 text-[13px] leading-none text-white',
                  'opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100',
                )}
              >
                ×
              </button>
            )}
          </div>
        ))}
      </div>
      {previewItem && (
        <Lightbox
          src={previewItem.url}
          alt={previewItem.name}
          caption={previewItem.name}
          onClose={() => setPreviewItem(null)}
        />
      )}
    </>
  );
}
