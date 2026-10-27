import {
  useEffect,
  useRef,
  useState,
  type ClipboardEvent,
  type DragEvent,
  type KeyboardEvent,
} from 'react';
import { cn } from '../cn';
import { AttachmentList, type AttachmentItem } from '../AttachmentList';
import { Select } from '../Select';

/**
 * AI 文本域 —— 聊天式输入区（Apple HIG 风格），由通用组件组装：
 * - AttachmentList：图片附件条（按钮选择 / 拖拽 / 粘贴上传由本组件提供）
 * - Select（ghost + 向上弹出）：模型选择
 * - 多行 textarea 自动增高（maxRows 封顶后内部滚动）
 * - Enter 发送、Shift+Enter 换行；中文输入法组词中的 Enter 不触发发送
 * - streaming 时发送按钮变停止按钮
 * 附件与模型状态由调用方管理（受控组件）。
 */

/** 附件条目（与 AttachmentItem 结构一致） */
export type ComposerAttachment = AttachmentItem;

export type ComposerModel = {
  id: string;
  name: string;
  /** 可选标注，如「默认」「推理」 */
  badge?: string;
};

export type ComposerProps = {
  value: string;
  onChange: (value: string) => void;
  onSend?: () => void;
  /** 生成中点停止 */
  onStop?: () => void;
  /** 生成中：发送按钮切换为停止按钮 */
  streaming?: boolean;
  disabled?: boolean;
  placeholder?: string;
  models?: ComposerModel[];
  model?: string;
  onModelChange?: (id: string) => void;
  attachments?: ComposerAttachment[];
  /** 追加文件回调（已过滤为图片），来源：按钮 / 拖拽 / 粘贴 */
  onAddFiles?: (files: File[]) => void;
  onRemoveAttachment?: (id: string) => void;
  /** 自动增高行数上限，默认 8 */
  maxRows?: number;
  className?: string;
};

const ArrowUpIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.5"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-4"
    aria-hidden
  >
    <path d="M12 19V5M5 12l7-7 7 7" />
  </svg>
);

const StopIcon = () => (
  <svg viewBox="0 0 24 24" fill="currentColor" className="size-3.5" aria-hidden>
    <rect x="6.5" y="6.5" width="11" height="11" rx="2" />
  </svg>
);

const ImageIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-[18px]"
    aria-hidden
  >
    <rect x="3" y="3" width="18" height="18" rx="4" />
    <circle cx="9" cy="9" r="1.8" />
    <path d="M21 15.5l-4.8-4.8L7 20" />
  </svg>
);

function IconButton({
  label,
  onClick,
  disabled,
  children,
}: {
  label: string;
  onClick?: () => void;
  disabled?: boolean;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      disabled={disabled}
      className={cn(
        'flex size-8 items-center justify-center rounded-full text-secondary-label transition-colors',
        'hover:bg-tertiary-fill hover:text-label',
        'disabled:pointer-events-none disabled:opacity-40',
      )}
    >
      {children}
    </button>
  );
}

export function Composer({
  value,
  onChange,
  onSend,
  onStop,
  streaming = false,
  disabled = false,
  placeholder = '输入消息…',
  models,
  model,
  onModelChange,
  attachments = [],
  onAddFiles,
  onRemoveAttachment,
  maxRows = 8,
  className,
}: ComposerProps) {
  const taRef = useRef<HTMLTextAreaElement>(null);
  const fileRef = useRef<HTMLInputElement>(null);
  const [dragOver, setDragOver] = useState(false);

  const canSend = !disabled && !streaming && (value.trim().length > 0 || attachments.length > 0);

  // 自动增高：现代浏览器用纯 CSS field-sizing:content（见 textarea 的 field-sizing-content 类），
  // 仅在不支持的旧浏览器降级为 JS 计算高度；行高与内边距从计算样式读取，随主题字号自适应
  useEffect(() => {
    if (typeof CSS !== 'undefined' && CSS.supports?.('field-sizing', 'content')) return;
    const ta = taRef.current;
    if (!ta) return;
    const cs = getComputedStyle(ta);
    const lh = Number.parseFloat(cs.lineHeight) || 20;
    const pad = Number.parseFloat(cs.paddingTop) + Number.parseFloat(cs.paddingBottom) || 0;
    ta.style.height = 'auto';
    ta.style.height = `${Math.min(ta.scrollHeight, maxRows * lh + pad)}px`;
  }, [value, maxRows]);

  const pickImages = (list: FileList | File[] | null | undefined) => {
    const files = Array.from(list ?? []).filter((f) => f.type.startsWith('image/'));
    if (files.length > 0) onAddFiles?.(files);
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key !== 'Enter' || e.shiftKey || e.nativeEvent.isComposing) return;
    e.preventDefault();
    if (canSend) onSend?.();
  };

  const handlePaste = (e: ClipboardEvent<HTMLTextAreaElement>) => {
    if (!onAddFiles) return;
    pickImages(e.clipboardData?.files);
  };

  const handleDrop = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setDragOver(false);
    if (!onAddFiles || disabled) return;
    pickImages(e.dataTransfer?.files);
  };

  return (
    <div
      className={cn(
        'rounded-2xl border border-separator bg-background transition-all duration-200 ease-out',
        'focus-within:border-tint/60 focus-within:shadow-card focus-within:ring-[3px] focus-within:ring-tint/10',
        dragOver && 'border-tint/60 ring-[3px] ring-tint/10',
        disabled && 'opacity-60',
        className,
      )}
      onDragOver={(e) => {
        e.preventDefault();
        if (!disabled) setDragOver(true);
      }}
      onDragLeave={() => setDragOver(false)}
      onDrop={handleDrop}
    >
      {/* 附件缩略图条 */}
      <AttachmentList items={attachments} onRemove={onRemoveAttachment} className="px-3 pt-3" />

      {/* 文本输入 */}
      <textarea
        ref={taRef}
        rows={1}
        value={value}
        disabled={disabled}
        placeholder={placeholder}
        aria-label="消息输入"
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        onPaste={handlePaste}
        style={{ maxHeight: `calc(${maxRows}lh + 16px)` }}
        className={cn(
          'field-sizing-content w-full resize-none bg-transparent px-3.5 pb-1 pt-3 text-sm text-label',
          'placeholder:text-placeholder-text focus:outline-none',
          'overflow-y-auto',
        )}
      />

      {/* 底部工具行：模型选择 · 图片 · 发送/停止 */}
      <div className="flex items-center gap-1.5 px-2.5 pb-2.5 pt-1">
        {models && models.length > 0 && (
          <Select
            variant="ghost"
            placement="top"
            ariaLabel="选择模型"
            value={model ?? null}
            onChange={onModelChange}
            options={models.map((m) => ({ value: m.id, label: m.name, badge: m.badge }))}
            placeholder="选择模型"
            disabled={disabled}
          />
        )}

        <div className="flex-1" />

        {onAddFiles && (
          <>
            <IconButton
              label="添加图片"
              onClick={() => fileRef.current?.click()}
              disabled={disabled}
            >
              <ImageIcon />
            </IconButton>
            <input
              ref={fileRef}
              type="file"
              accept="image/*"
              multiple
              className="hidden"
              onChange={(e) => {
                pickImages(e.target.files);
                e.target.value = '';
              }}
            />
          </>
        )}

        {streaming ? (
          <button
            type="button"
            aria-label="停止生成"
            onClick={onStop}
            className={cn(
              'flex size-8 items-center justify-center rounded-full bg-label text-background',
              'transition-transform hover:scale-105 active:scale-95',
            )}
          >
            <StopIcon />
          </button>
        ) : (
          <button
            type="button"
            aria-label="发送"
            disabled={!canSend}
            onClick={() => canSend && onSend?.()}
            className={cn(
              'flex size-8 items-center justify-center rounded-full bg-tint text-white',
              'transition-all duration-200 hover:opacity-90 active:scale-95',
              'disabled:pointer-events-none disabled:opacity-30',
            )}
          >
            <ArrowUpIcon />
          </button>
        )}
      </div>
    </div>
  );
}
