import { useCallback, useEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { cn } from '../cn';

/*
 * Message / MessageList —— 对话骨架（混合式布局）
 *
 * 规则（Apple HIG / emil-design-eng）：
 * - user 消息右对齐 tint 气泡（iMessage 式）；assistant 消息左对齐通栏，长内容（Markdown/工具卡）不受气泡挤压
 * - 组件零业务逻辑：消息数据、复制/重新生成的行为都由调用方决定，这里只给回调插槽
 * - loading 态 = 三点呼吸（Tailwind 内置 animate-pulse + delay 错开），reduced-motion 下退化为静止点
 * - MessageList 滚动跟随用 ref 而非 state 判断贴底，避免"新内容到达 → state 滞后一帧 → 跟丢"
 */

export type MessageRole = 'user' | 'assistant';

export interface MessageProps {
  role: MessageRole;
  /** assistant 头部名字，默认 'ARK' */
  name?: string;
  /** 头部时间戳（可选），如 '14:32' */
  timestamp?: string;
  /** assistant 等待首 token 的 loading 态（三点呼吸） */
  loading?: boolean;
  /** assistant 操作条：复制（传入即显示） */
  onCopy?: () => void;
  /** assistant 操作条：重新生成（传入即显示） */
  onRegenerate?: () => void;
  className?: string;
  /** 消息内容；loading 态下可省略 */
  children?: ReactNode;
}

const CopyIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3.5"
    aria-hidden
  >
    <rect x="9" y="9" width="12" height="12" rx="2" />
    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
  </svg>
);

const RotateIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3.5"
    aria-hidden
  >
    <path d="M3 12a9 9 0 1 0 2.6-6.4L3 8" />
    <path d="M3 3v5h5" />
  </svg>
);

function ActionButton({
  label,
  onClick,
  children,
}: {
  label: string;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-label={label}
      title={label}
      className="flex size-7 items-center justify-center rounded-md text-tertiary-label transition-colors duration-150 ease-out hover:bg-tertiary-fill hover:text-label"
    >
      {children}
    </button>
  );
}

export function Message({
  role,
  name = 'ARK',
  timestamp,
  loading = false,
  onCopy,
  onRegenerate,
  className,
  children,
}: MessageProps) {
  if (role === 'user') {
    return (
      <div className={cn('flex justify-end', className)}>
        <div className="max-w-[80%] rounded-2xl rounded-br-md bg-tint px-4 py-2.5 text-[15px] leading-relaxed text-white">
          {children}
        </div>
      </div>
    );
  }

  return (
    <div className={cn('flex flex-col gap-1.5', className)}>
      <div className="flex items-center gap-2">
        <span className="size-2 rounded-full bg-tint" aria-hidden />
        <span className="text-xs font-semibold text-secondary-label">{name}</span>
        {timestamp && <span className="text-xs text-tertiary-label">{timestamp}</span>}
      </div>

      {loading ? (
        <div className="flex items-center gap-1 py-1.5" role="status" aria-label="正在生成回复">
          {[0, 1, 2].map((i) => (
            <span
              key={i}
              className="size-1.5 animate-pulse rounded-full bg-tertiary-label"
              style={{ animationDelay: `${i * 150}ms` }}
            />
          ))}
        </div>
      ) : (
        <div className="text-[15px] leading-relaxed text-label">{children}</div>
      )}

      {(onCopy || onRegenerate) && !loading && (
        <div className="flex items-center gap-1">
          {onCopy && (
            <ActionButton label="复制" onClick={onCopy}>
              <CopyIcon />
            </ActionButton>
          )}
          {onRegenerate && (
            <ActionButton label="重新生成" onClick={onRegenerate}>
              <RotateIcon />
            </ActionButton>
          )}
        </div>
      )}
    </div>
  );
}

export interface MessageListProps {
  children: ReactNode;
  className?: string;
  /** 距底部小于该值(px)视为"贴底"，新内容自动跟随，默认 48 */
  stickThreshold?: number;
  /** 距底部超过该值(px)显示"回到底部"按钮，默认 160 */
  jumpThreshold?: number;
}

const ArrowDownIcon = () => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-4"
    aria-hidden
  >
    <path d="M12 5v14" />
    <path d="m19 12-7 7-7-7" />
  </svg>
);

/**
 * 消息流容器：贴底自动跟随新内容，上滑后出现"回到底部"悬浮按钮。
 * 父容器需有确定高度（如 flex-1 min-h-0 或 h-[480px]）。
 */
export function MessageList({
  children,
  className,
  stickThreshold = 48,
  jumpThreshold = 160,
}: MessageListProps) {
  // scroll-state() 容器查询（Chrome 133+）在 CSS 层驱动按钮显隐（见 styles/components.css），
  // 不支持的引擎降级为滚动监听 + setState；贴底判定（stickRef）供跟随逻辑用，两条路径都要
  const cssScrollState =
    typeof CSS !== 'undefined' && !!CSS.supports?.('container-type', 'scroll-state');
  const scrollRef = useRef<HTMLDivElement>(null);
  /** 贴底判定放 ref：children 更新的 effect 里直接读，不吃 state 滞后 */
  const stickRef = useRef(true);
  const [distFromBottom, setDistFromBottom] = useState(0);

  const handleScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    const dist = el.scrollHeight - el.scrollTop - el.clientHeight;
    stickRef.current = dist <= stickThreshold;
    if (!cssScrollState) setDistFromBottom(dist); // CSS 路径下按钮显隐不经过 state，滚动零重渲染
  }, [stickThreshold, cssScrollState]);

  // 内容更新（children 引用变化）时，若仍贴底则跟随滚到最新
  useEffect(() => {
    const el = scrollRef.current;
    if (el && stickRef.current) el.scrollTop = el.scrollHeight;
  }, [children]);

  const scrollToBottom = () => {
    const el = scrollRef.current;
    if (!el) return;
    el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' });
    stickRef.current = true;
    setDistFromBottom(0);
  };

  const showJump = distFromBottom > jumpThreshold;

  return (
    <div className={cn('relative h-full', className)}>
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        style={{ containerName: 'msglist', containerType: 'scroll-state' }}
        className="h-full overflow-y-auto overscroll-contain"
      >
        <div className="flex flex-col gap-5 py-1">{children}</div>
      </div>

      <div
        className={cn(
          'pointer-events-none absolute inset-x-0 bottom-3 flex justify-center transition-opacity duration-200 ease-out',
          cssScrollState ? 'msglist-jump opacity-0' : showJump ? 'opacity-100' : 'opacity-0',
        )}
      >
        <button
          type="button"
          onClick={scrollToBottom}
          aria-label="回到底部"
          className={cn(
            'flex size-8 items-center justify-center rounded-full bg-secondary-grouped-background text-secondary-label shadow-card transition-colors duration-150 ease-out hover:text-label',
            !cssScrollState && 'pointer-events-auto', // CSS 路径下由容器查询规则控制可点性
          )}
        >
          <ArrowDownIcon />
        </button>
      </div>
    </div>
  );
}
