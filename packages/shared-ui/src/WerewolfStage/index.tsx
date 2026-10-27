import type { ReactNode } from 'react';
import { cn } from '../cn';
import { VStack } from '../layout/Stack';
import { PlayerSeat, type WerewolfPlayer } from './PlayerSeat';

export type { WerewolfPlayer, PlayerSeatProps } from './PlayerSeat';
export { PlayerSeat } from './PlayerSeat';
export { RoleBadge, type WerewolfRoleTone, type RoleBadgeProps } from './RoleBadge';

export type WerewolfStageProps = {
  /** 舞台大标题（如"AI 狼人杀"） */
  title?: string;
  /** 座位列表：按座位号排序，前半左列、后半右列 */
  players: WerewolfPlayer[];
  /** 对局阶段：夜晚叠蓝黑夜幕遮罩；缺省不加横幅不遮罩 */
  phase?: 'night' | 'day';
  /** 阶段横幅文案（如"天黑请闭眼""第 2 天 · 白天发言"）；缺省用内置文案 */
  phaseLabel?: ReactNode;
  /** 底部大字幕（旁白 / 天亮宣言） */
  caption?: ReactNode;
  /** 背景图（森林插画等），cover 铺满；缺省为深色渐变底 */
  backgroundUrl?: string;
  /** 全屏沉浸：撑满视口、去圆角（整页即舞台时用） */
  fullBleed?: boolean;
  /** 舞台中央内容：发言流 / 白天记录 / 交互面板 */
  children?: ReactNode;
  className?: string;
};

const PHASE_LABEL: Record<NonNullable<WerewolfStageProps['phase']>, string> = {
  night: '🌙 天黑请闭眼',
  day: '☀️ 天亮请睁眼',
};

/**
 * 狼人杀舞台 —— 沉浸式对局画面：左右座位列 + 中央内容 + 底部大字幕。
 * 纯展示组件：对局状态由调用方下发（引擎快照 → 视角裁剪 → 本组件渲染）。
 */
export function WerewolfStage({
  title,
  players,
  phase,
  phaseLabel,
  caption,
  backgroundUrl,
  fullBleed = false,
  children,
  className,
}: WerewolfStageProps) {
  const ordered = players.toSorted((a, b) => a.seat - b.seat);
  const leftCount = Math.ceil(ordered.length / 2);
  const left = ordered.slice(0, leftCount);
  const right = ordered.slice(leftCount);

  return (
    <div
      className={cn(
        'relative flex flex-col overflow-hidden',
        fullBleed ? 'h-dvh min-h-0 rounded-none' : 'min-h-125 rounded-2xl',
        'bg-gradient-to-b from-gray-6 to-gray-3 dark:from-gray-2 dark:to-black',
        className,
      )}
    >
      {backgroundUrl && (
        <img
          src={backgroundUrl}
          alt=""
          aria-hidden
          className="absolute inset-0 size-full object-cover brightness-[0.6]"
        />
      )}

      {/* 夜幕遮罩：夜晚压暗偏蓝，白天淡出（只动 opacity） */}
      <div
        aria-hidden
        className={cn(
          'absolute inset-0 bg-gradient-to-b from-blue/25 via-black/20 to-black/40 transition-opacity duration-700',
          phase === 'night' ? 'opacity-100' : 'opacity-0',
        )}
      />

      <div
        className={cn(
          'relative flex flex-1 items-stretch justify-between gap-4',
          fullBleed ? 'p-[3vmin]' : 'p-6',
        )}
      >
        {/* 左右列：固定等宽，座位居中成组、固定间隙（避免满屏拉散） */}
        <VStack gap={12} align="start" justify="center" className="w-[168px] shrink-0">
          {left.map((player) => (
            <PlayerSeat key={player.seat} {...player} side="left" />
          ))}
        </VStack>

        {/* 中央：标题 + 阶段横幅 + 字幕槽 + 内容。字幕槽恒高，出现/消失不引起内容区跳动 */}
        <VStack gap={4} align="center" className="min-w-0 flex-1 self-stretch">
          {title && (
            <h2 className="text-center text-3xl font-black tracking-widest text-yellow drop-shadow-lg">
              {title}
            </h2>
          )}
          {phase && (
            <div
              className={cn(
                'rounded-full px-4 py-1 text-sm font-bold shadow-card',
                phase === 'night' ? 'bg-blue/85 text-white' : 'bg-yellow/90 text-black',
              )}
            >
              {phaseLabel ?? PHASE_LABEL[phase]}
            </div>
          )}
          <div className="flex h-14 shrink-0 items-center justify-center">
            {caption && (
              <p className="animate-fade-in text-xl font-bold text-white drop-shadow">{caption}</p>
            )}
          </div>
          {children && <div className="min-h-0 flex-1">{children}</div>}
        </VStack>

        {/* 右列 */}
        <VStack gap={12} align="end" justify="center" className="w-[168px] shrink-0">
          {right.map((player) => (
            <PlayerSeat key={player.seat} {...player} side="right" />
          ))}
        </VStack>
      </div>
    </div>
  );
}
