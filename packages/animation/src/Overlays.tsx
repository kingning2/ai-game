/**
 * 对局过场浮层：开局倒计时（5-4-3-2-1）与天黑揭示（狼嚎 + GSAP 时间线渐隐）。
 * 自 apps/web/werewolf/arena 收编进本包：所有过场动画组件统一住在 animation 包。
 * token-guard:allow —— drop-shadow 里的 rgb() 系搬移前的既有代码，非新增硬编码。
 */
import { useRef } from 'react';
import { useGSAP } from '@gsap/react';
import gsap from 'gsap';

/** 开局倒计时浮层：5-4-3-2-1 逐秒过场（stage 就绪前在等待页展示，倒数完才开演） */
export function CountdownOverlay({ value, waiting }: { value: number; waiting: boolean }) {
  return (
    <div className="absolute inset-0 z-30 flex flex-col items-center justify-center gap-6 bg-arena/70 backdrop-blur-sm">
      <p className="animate-fade-in text-lg font-medium text-arena-muted">
        对局即将开始，请各位选手就位
      </p>
      <p
        key={value}
        className="animate-zoom-in text-8xl font-black tracking-wider text-arena-text drop-shadow-[0_4px_24px_rgb(0_0_0/0.6)]"
      >
        {value > 0 ? value : '开始！'}
      </p>
      <p className="animate-fade-in text-sm text-arena-muted">
        {waiting ? '选手正在入场，主持人整理牌局中……' : '主持人即将入场 · 天黑请闭眼……'}
      </p>
    </div>
  );
}

/** 「天黑请闭眼」逐字过场标题 */
const NIGHTFALL_TITLE = ['天', '黑', '请', '闭', '眼'];

/**
 * 天黑过场浮层（GSAP 时间线，约 4.7s，与 5.5s 狼嚎对齐）：
 *   熄灯 0.35s → 月亮升起 + 金晕 → 标题逐字浮现 → 金线展开 → 驻留 1s → 黑幕推开揭示舞台。
 * 只动 opacity / transform（合成器属性）；只动 scale(≥0.9) 不从 0 起。
 * reduced-motion：跳过时间线静态展示，由父级兜底定时收场。
 * 卸载 / 中途离页由 useGSAP 自动 revert，不需要手动 cleanup。
 */
export function NightfallOverlay({ onDone }: { onDone: () => void }) {
  const root = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
      const q = gsap.utils.selector(root);
      gsap
        .timeline({ onComplete: onDone })
        // 熄灯：黑幕快速压上，遮住倒计时余像
        .fromTo(root.current, { opacity: 0 }, { opacity: 1, duration: 0.35, ease: 'power2.in' })
        // 月亮升起：金晕先亮，月亮从下方浮起
        .from(q('[data-glow]'), { opacity: 0, duration: 0.7, ease: 'power1.out' }, '+=0.25')
        .from(
          q('[data-moon]'),
          { opacity: 0, y: 24, scale: 0.92, duration: 0.7, ease: 'power2.out' },
          '<0.1',
        )
        // 标题逐字浮现（狼嚎第一声正落在这段）
        .from(
          q('[data-char]'),
          { opacity: 0, y: 10, duration: 0.45, stagger: 0.08, ease: 'power2.out' },
          '-=0.15',
        )
        // 金色发丝线自中心展开
        .from(q('[data-rule]'), { scaleX: 0, duration: 0.5, ease: 'power2.out' }, '-=0.3')
        // 驻留片刻，然后黑幕推开（轻微放大 = 镜头缓缓撤出黑暗）
        .to(
          root.current,
          { opacity: 0, scale: 1.04, duration: 1.4, ease: 'power2.inOut' },
          '+=1.0',
        );
    },
    { scope: root, dependencies: [onDone] },
  );

  return (
    <div
      ref={root}
      aria-hidden
      className="pointer-events-none absolute inset-0 z-30 flex flex-col items-center justify-center gap-5 bg-arena"
    >
      {/* 金晕：纯令牌色 + blur，只参与 opacity 动画 */}
      <div className="relative flex flex-col items-center gap-5">
        <div
          data-glow
          className="absolute top-1/2 left-1/2 h-44 w-44 -translate-x-1/2 -translate-y-1/2 rounded-full bg-arena-gold/15 blur-3xl"
        />
        <span data-moon className="relative text-7xl">
          🌙
        </span>
        <p className="relative flex gap-3 text-3xl font-medium text-arena-text">
          {NIGHTFALL_TITLE.map((char) => (
            <span key={char} data-char>
              {char}
            </span>
          ))}
        </p>
        <div data-rule className="relative h-px w-28 bg-arena-gold/60" />
      </div>
    </div>
  );
}

/** 天黑过场兜底上限：GSAP 时间线约 4.7s 会回调 onDone 提前收场，此值仅兜底（含 reduced-motion 静态展示） */
export const NIGHTFALL_MS = 5400;
