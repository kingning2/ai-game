import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from 'react';
import { cn } from '../cn';

export type TypewriterHandle = {
  /** 跳过动画：立即显示全部已接收文本（配合「跳过」按钮） */
  skip: () => void;
  /** 命令式喟入一个 chunk（非受控用法：不传 text prop，在 SSE onmessage 回调里直接调） */
  push: (chunk: string) => void;
  /** 清空缓冲、进度归零（非受控用法的重置） */
  reset: () => void;
};

export type TypewriterProps = {
  /**
   * 目标文本（受控用法）。流式场景：SSE token 到达时对同一 state 追加后再传进来，
   * 组件会从上次进度继续打（prop 增长即续打，无需重置）。
   * 也可不传 text 走非受控：用 ref.push(chunk) 逐段喟入；两者同时存在时以 text 为准。
   */
  text?: string;
  /** 打字速度：每秒字符数（默认 60，接近流畅阅读的输出节奏） */
  speed?: number;
  /** 显示打字光标（默认 true；text 为空时光标持续闪烁 = 等待首 token） */
  cursor?: boolean;
  /** 打完后隐藏光标（默认 true；传 false 让光标常驻，适合多轮对话光标停在结尾） */
  hideCursorOnDone?: boolean;
  /** 已接收文本全部显示完毕时触发；流式追加后再次打完会再次触发 */
  onFinished?: () => void;
  className?: string;
};

const TICK_MS = 16;

/**
 * 流式打字机 —— AI 回复展示：
 * - 16ms tick + 浮点累加器：speed 不足 1 字/tick 时均摊推进，节奏均匀
 * - 渲染只做字符串切片（无逐字 span），几千字的长回复也不掉帧
 * - 跳过用 Number.MAX_SAFE_INTEGER + 渲染 clamp：批处理下「跳过 + 新文本」同时 setState 也正确
 * - 无障碍：aria-label 读全文（屏幕阅读器不逐字播报），可见切片 aria-hidden；
 *   prefers-reduced-motion 时直接显示全文
 */
export const Typewriter = forwardRef<TypewriterHandle, TypewriterProps>(function Typewriter(
  { text, speed = 60, cursor = true, hideCursorOnDone = true, onFinished, className },
  ref,
) {
  const [count, setCount] = useState(0);
  const [reduced, setReduced] = useState(false);
  const [buffer, setBuffer] = useState('');
  const accRef = useRef(0);
  const doneRef = useRef(false);

  // 受控（text prop）与非受控（push 喂入 buffer）二选一，同时存在时以 text 为准
  const source = text ?? buffer;

  // 无障碍：偏好减弱动效 → 跳过逐字动画直接显示全文
  useEffect(() => {
    const mq = matchMedia('(prefers-reduced-motion: reduce)');
    const sync = () => setReduced(mq.matches);
    sync();
    mq.addEventListener('change', sync);
    return () => mq.removeEventListener('change', sync);
  }, []);

  // 打字推进：打完即停（不空转），text 增长后自动续打；
  // reduced-motion 不开循环，渲染层直接派生全文（不在 effect 里 setState）。
  // 注：重发新文本请换 key 重挂载（流式追加只增不减则无需处理）
  useEffect(() => {
    if (reduced || count >= source.length) return;
    const safeSpeed = Math.max(1, speed);
    const timer = setInterval(() => {
      accRef.current += (safeSpeed * TICK_MS) / 1000;
      const inc = Math.floor(accRef.current);
      if (inc <= 0) return;
      accRef.current -= inc;
      setCount((c) => Math.min(source.length, c + inc));
    }, TICK_MS);
    return () => clearInterval(timer);
  }, [count, source.length, speed, reduced, source]);

  // 完成通知（仅一次；流式追加后重新打完会再次触发）
  useEffect(() => {
    if (count >= source.length) {
      if (!doneRef.current && source.length > 0) {
        doneRef.current = true;
        accRef.current = 0;
        onFinished?.();
      }
    } else {
      doneRef.current = false;
    }
  }, [count, source.length, onFinished]);

  useImperativeHandle(
    ref,
    () => ({
      skip: () => setCount(Number.MAX_SAFE_INTEGER),
      push: (chunk: string) => setBuffer((b) => b + chunk),
      reset: () => {
        setBuffer('');
        setCount(0);
        accRef.current = 0;
        doneRef.current = false;
      },
    }),
    [],
  );

  const done = source.length > 0 && (reduced || count >= source.length);
  const shown = reduced ? source : source.slice(0, Math.min(count, source.length));
  const showCursor = cursor && !reduced && (!done || !hideCursorOnDone);

  return (
    <p aria-label={source} className={cn('whitespace-pre-wrap text-sm text-label', className)}>
      <span aria-hidden>{shown}</span>
      {showCursor && (
        <span
          aria-hidden
          className="animate-caret-blink ms-0.5 inline-block h-[1.1em] w-[2px] translate-y-[0.18em] rounded-full bg-tint"
        />
      )}
    </p>
  );
});
