/**
 * OpeningCeremony —— 开局发牌仪式（GSAP 时间线，全程零文字提示）：
 *   入座显形 → 洗牌（卡片交替轻摆）→ 发牌（牌背从场心飞向各座）→ 亮牌（牌背翻出身份）→ 亮渐暗。
 * 收场回调 onComplete 后由调用方接倒计时（暗屏数字）→ NightfallOverlay（由暗到亮揭示舞台）。
 *
 * 数据契约：roster（座位名册）就绪即可渲染静场；players（真实发牌）全部就绪才触发时间线——
 * 亮牌内容必须与对局一致（server seat_roles 事实源），未就绪前只演入座，不发牌。
 * 动画只动 opacity / transform；reduced-motion 跳过演出直接 onComplete。
 */
import { useRef } from 'react';
import { useGSAP } from '@gsap/react';
import gsap from 'gsap';
import { Avatar, cn } from '@agent-packages/shared-ui';

/** 座位名册（入座静场阶段即可下发） */
export type CeremonyRoster = {
  seat: number;
  name: string;
  avatarSrc?: string;
};

/** 发牌结果：身份全部就绪后由调用方整体下发，触发发牌/亮牌时间线 */
export type CeremonyPlayer = CeremonyRoster & {
  roleLabel: string;
  roleTone: 'wolf' | 'good';
};

export type OpeningCeremonyProps = {
  roster: CeremonyRoster[];
  /** 真实发牌；null = 身份未就绪，只演入座静场等数据 */
  players: CeremonyPlayer[] | null;
  /** 时间线走完（亮渐暗收黑）回调；reduced-motion 时立即回调 */
  onComplete: () => void;
  /** 舞台背景图（与对局页同款） */
  backgroundUrl?: string;
};

/** 身份牌色调：与舞台 RoleBadge 语义一致（狼红 / 好人金） */
const TONE_CLASS: Record<CeremonyPlayer['roleTone'], string> = {
  wolf: 'bg-red text-on-accent',
  good: 'bg-yellow/90 text-stage-ink-strong',
};

/** 洗牌轻摆的交替方向（查表替代三元） */
const SWAY_Y = [-5, 5];
const SWAY_ROTATION = [2.5, -2.5];

export function OpeningCeremony({
  roster,
  players,
  onComplete,
  backgroundUrl,
}: OpeningCeremonyProps) {
  const root = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      if (!players) return; // 身份未就绪：只渲染入座静场，不发牌
      if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
        onComplete();
        return;
      }
      const q = gsap.utils.selector(root);

      // 发牌位移：牌背从场心飞向各座（effect 内测 DOM 几何，非 render 路径）
      const stageRect = root.current?.getBoundingClientRect();
      const centerX = (stageRect?.width ?? 0) / 2;
      const centerY = (stageRect?.height ?? 0) / 2;
      const flyDx: number[] = [];
      const flyDy: number[] = [];
      for (const el of q('[data-chip]')) {
        const rect = el.getBoundingClientRect();
        flyDx.push(centerX - (rect.left + rect.width / 2));
        flyDy.push(centerY - (rect.top + rect.height / 2));
      }

      gsap
        .timeline({ onComplete })
        // ① 入座显形：选手卡自下方轻微上浮，依次落位
        .from(q('[data-card]'), {
          opacity: 0,
          y: 14,
          duration: 0.5,
          stagger: 0.06,
          ease: 'power2.out',
        })
        // ② 洗牌：卡片交替轻摆两轮（只动 transform）
        .to(q('[data-card]'), {
          y: (i: number) => SWAY_Y[i % SWAY_Y.length],
          rotation: (i: number) => SWAY_ROTATION[i % SWAY_ROTATION.length],
          duration: 0.4,
          stagger: 0.08,
          yoyo: true,
          repeat: 3,
          ease: 'sine.inOut',
        })
        .set(q('[data-card]'), { y: 0, rotation: 0 })
        // ③ 发牌：牌背自场心飞向各座（位移用预计算数组按序注入），依次到位
        .fromTo(
          q('[data-chip]'),
          {
            opacity: 0,
            scale: 0.4,
            x: (i: number) => flyDx[i] ?? 0,
            y: (i: number) => flyDy[i] ?? 0,
          },
          {
            opacity: 1,
            scale: 1,
            x: 0,
            y: 0,
            duration: 0.45,
            stagger: 0.12,
            ease: 'power2.out',
          },
          '-=0.2',
        )
        // ④ 亮牌：牌背转出、身份牌转入，逐座翻开
        .to(
          q('[data-chip-back]'),
          { opacity: 0, rotateY: 90, duration: 0.3, stagger: 0.15, ease: 'power2.in' },
          '+=0.15',
        )
        .fromTo(
          q('[data-chip-front]'),
          { opacity: 0, rotateY: -90 },
          { opacity: 1, rotateY: 0, duration: 0.3, stagger: 0.15, ease: 'power2.out' },
          '<0.12',
        )
        // ⑤ 全员亮牌驻留片刻 → 亮渐暗收黑（交棒给倒计时）
        .to(q('[data-veil]'), { opacity: 1, duration: 1.1, ease: 'power2.inOut' }, '+=0.9');
    },
    { scope: root, dependencies: [players, onComplete] },
  );

  return (
    <div ref={root} className="relative h-dvh overflow-hidden bg-arena">
      {backgroundUrl && (
        <img
          src={backgroundUrl}
          alt=""
          aria-hidden
          className="absolute inset-0 size-full object-cover brightness-[0.6]"
        />
      )}
      {/* 夜幕遮罩：与舞台夜晚同款压暗偏蓝 */}
      <div
        aria-hidden
        className="absolute inset-0 bg-gradient-to-b from-blue/25 via-black/20 to-black/40"
      />

      <div className="relative mx-auto grid h-full max-w-5xl grid-cols-3 content-center gap-3 px-6 sm:grid-cols-4">
        {roster.map((member) => {
          const dealt = players?.find((p) => p.seat === member.seat);
          return (
            <div
              key={member.seat}
              data-card
              className="flex flex-col items-center gap-2 rounded-2xl bg-arena-raised/60 p-3 ring-1 ring-arena-line"
            >
              <Avatar src={member.avatarSrc} fallback={member.name.slice(0, 1)} size="lg" />
              <span className="max-w-28 truncate text-xs font-medium text-arena-text">
                {member.seat} 号 · {member.name}
              </span>
              {/* 牌槽恒高：发牌前后布局不跳 */}
              <div className="relative flex h-7 w-16 items-center justify-center">
                {dealt && (
                  <>
                    <div
                      data-chip
                      data-chip-back
                      className="absolute inset-0 flex items-center justify-center rounded-md bg-arena-gold/20 opacity-0 ring-1 ring-arena-gold/70"
                    >
                      <span className="block size-2 rotate-45 bg-arena-gold/80" />
                    </div>
                    <div
                      data-chip-front
                      className={cn(
                        'absolute inset-0 flex items-center justify-center rounded-md px-1 text-xs font-bold opacity-0',
                        TONE_CLASS[dealt.roleTone],
                      )}
                    >
                      {dealt.roleLabel}
                    </div>
                  </>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {/* 收黑幕：亮渐暗的最后一段，遮住向倒计时页的切换 */}
      <div
        data-veil
        aria-hidden
        className="pointer-events-none absolute inset-0 bg-arena opacity-0"
      />
    </div>
  );
}
