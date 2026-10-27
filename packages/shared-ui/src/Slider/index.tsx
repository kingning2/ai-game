import { useCallback, useRef, type KeyboardEvent, type PointerEvent } from 'react';
import { cn } from '../cn';

export type SliderProps = {
  /** 当前值（受控） */
  value?: number;
  min?: number;
  max?: number;
  step?: number;
  onChange?: (value: number) => void;
  'aria-label'?: string;
  className?: string;
};

const clamp = (v: number, min: number, max: number) => Math.min(max, Math.max(min, v));

/**
 * 滑块 —— iOS UISlider 规格（唯一获取数值的输入件；展示用 Progress，职责分离）：
 * - 细轨道（4px）：左侧 tint 填充走逻辑属性（start-0，rtl 自动反向），右侧 fill
 * - 滑块 28px 白色圆（insetInlineStart 百分比 + translate 居中，方向自适应），
 *   按住 spring 放大；拖出轨道靠 pointer capture 继续跟随
 * - 键盘：方向键 ±step、Home/End 到边界；touch-none 防止拖动被页面滚动打断
 */
export function Slider({
  value = 0,
  min = 0,
  max = 100,
  step = 1,
  onChange,
  'aria-label': ariaLabel,
  className,
}: SliderProps) {
  const trackRef = useRef<HTMLDivElement>(null);
  const pct = ((clamp(value, min, max) - min) / (max - min)) * 100;

  const setFromClientX = useCallback(
    (clientX: number) => {
      const rect = trackRef.current?.getBoundingClientRect();
      if (!rect || rect.width === 0) return;
      const ratio = clamp((clientX - rect.left) / rect.width, 0, 1);
      const snapped = Math.round((min + ratio * (max - min)) / step) * step;
      // 浮点步进累计误差（如 0.1+0.2）归一到 10 位小数
      onChange?.(clamp(Number(snapped.toFixed(10)), min, max));
    },
    [min, max, step, onChange],
  );

  const onPointerDown = (e: PointerEvent<HTMLDivElement>) => {
    e.currentTarget.setPointerCapture(e.pointerId);
    setFromClientX(e.clientX);
  };

  // 捕获期间跟随移动（含拖出轨道），松开即停
  const onPointerMove = (e: PointerEvent<HTMLDivElement>) => {
    if (e.currentTarget.hasPointerCapture(e.pointerId)) setFromClientX(e.clientX);
  };

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    const coarse = { ArrowRight: step, ArrowUp: step, ArrowLeft: -step, ArrowDown: -step } as const;
    if (e.key in coarse) {
      e.preventDefault();
      onChange?.(clamp(value + coarse[e.key as keyof typeof coarse], min, max));
    } else if (e.key === 'Home') {
      e.preventDefault();
      onChange?.(min);
    } else if (e.key === 'End') {
      e.preventDefault();
      onChange?.(max);
    }
  };

  return (
    <div
      ref={trackRef}
      role="slider"
      tabIndex={0}
      aria-label={ariaLabel}
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuenow={value}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onKeyDown={onKeyDown}
      className={cn(
        'relative flex h-7 w-full touch-none items-center',
        'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-tint',
        className,
      )}
    >
      {/* 细轨道：tint 填充从 start 侧生长，rtl 自动反向 */}
      <div className="relative h-1 w-full rounded-full bg-fill">
        <div
          className="absolute inset-y-0 start-0 rounded-full bg-tint"
          style={{ width: `${pct}%` }}
        />
      </div>
      {/* 滑块：百分比定位 + translate 居中；按住 spring 放大 */}
      <div
        className={cn(
          'pointer-events-none absolute size-7 rounded-full bg-white shadow-card',
          'transition-transform duration-300 ease-spring active:scale-110',
        )}
        style={{ insetInlineStart: `${pct}%`, translate: '-50% 0' }}
      />
    </div>
  );
}
