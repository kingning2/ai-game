import type { ReactNode } from 'react';
import { cn } from '../cn';
import { Avatar } from '../Avatar';
import { RoleBadge, type WerewolfRoleTone } from './RoleBadge';

/** 座位数据（上帝视角：身份由调用方下发） */
export type WerewolfPlayer = {
  /** 座位号，1 起 */
  seat: number;
  name: string;
  /** 头像图（AI 选手 logo） */
  avatarSrc?: string;
  /** 无图回退（emoji / 首字母） */
  avatarFallback?: string;
  roleLabel: string;
  roleTone?: WerewolfRoleTone;
  /** 默认 true；false 时置灰压暗 + 大号红叉盖头像（叉本身保持鲜红） */
  alive?: boolean;
  /** 正在发言：全彩浮出；其余座位压暗退后 */
  speaking?: boolean;
  /** 夜晚睁眼：全彩 + 金色呼吸光环 */
  spotlight?: boolean;
  /** 深度压暗（夜晚闭眼/旁观）——淘汰态时被忽略 */
  dimmed?: boolean;
  /** 头像覆盖层：刀/盾/查验结果等标记，挂载时 zoom-in 弹入 */
  overlay?: ReactNode;
};

export type PlayerSeatProps = WerewolfPlayer & {
  /** 座位所在侧：身份标签始终朝舞台中央（left 列标签在右，right 列整卡镜像） */
  side?: 'left' | 'right';
  className?: string;
};

/**
 * 单个座位：名字 + 座位号 + 头像 + 竖排身份标签。
 * 视觉态优先级：淘汰 > 发言/睁眼（全彩提层） > 未发言压暗 > 夜晚闭眼深度压暗。
 *
 * 实现约束：
 * - 状态类挂在子元素级（父级 grayscale/blend/filter 会传染红叉且无法撤销）
 * - 座位行不能挂任何 filter/transform/opacity（drop-shadow 移到了子元素），
 *   否则创建 stacking context 会隔离子元素的滤镜与混合效果
 */
export function PlayerSeat({
  seat,
  name,
  avatarSrc,
  avatarFallback,
  roleLabel,
  roleTone = 'good',
  alive = true,
  speaking = false,
  spotlight = false,
  dimmed = false,
  overlay,
  side = 'left',
  className,
}: PlayerSeatProps) {
  // 状态层：发言/睁眼=全彩无滤镜；未发言=brightness 压暗降饱和（暗底上不能用 multiply，
  // 乘法会把亮色直接吞进背景变成"消失"）；闭眼=深度压暗；淘汰=置灰+强压暗+降透明
  const stateClass = !alive
    ? 'opacity-35 grayscale brightness-50'
    : speaking || spotlight
      ? undefined
      : dimmed
        ? 'brightness-[0.4] saturate-75'
        : 'brightness-[0.55] saturate-[0.85]';

  return (
    <div
      className={cn(
        'relative transition-[transform] duration-300 ease-spring',
        // 发言聚光：轻微放大提层
        speaking && alive && 'z-10 scale-105',
        className,
      )}
    >
      <div className={cn('flex w-full items-center gap-2', side === 'right' && 'flex-row-reverse')}>
        <div
          className={cn(
            // 定宽：名字长短不影响头像/徽章的 x 坐标，左右列才能镜像对齐
            'flex w-[80px] flex-col items-center gap-1 drop-shadow-stage-soft transition-[opacity,filter] duration-300',
            stateClass,
          )}
        >
          <span
            className={cn(
              'w-full truncate text-center text-xs font-medium text-yellow',
              // 黑色双影描边：亮背景（月亮/雪地）上黄字仍可读，视频字幕惯例
              'text-shadow-stage-name',
            )}
          >
            {name}
          </span>
          <span
            className={cn(
              'flex size-6 items-center justify-center rounded-full text-xs font-medium text-stage-ink-strong',
              alive ? 'bg-yellow' : 'bg-gray-4',
            )}
          >
            {seat}
          </span>
        </div>
        <div className="relative">
          <Avatar
            src={avatarSrc}
            fallback={avatarFallback ?? name.slice(0, 1)}
            size="lg"
            className={cn(
              'drop-shadow-stage-soft transition-[opacity,filter] duration-300',
              stateClass,
              // 睁眼光环：金色描边 + 呼吸（只动 opacity）
              spotlight && alive && 'ring-2 ring-yellow/90 animate-pulse',
            )}
          />
          {/* 动作标记：刀/盾/查验结果，弹入动画（scale 0.95 起步，禁 scale(0)） */}
          {overlay && (
            <div className="absolute -right-2 -bottom-1 z-20 animate-zoom-in ease-spring">
              {overlay}
            </div>
          )}
          {/* 已淘汰：大号红叉盖在头像上（挂无混合层，保持鲜红） */}
          {!alive && (
            <svg
              viewBox="0 0 24 24"
              role="img"
              aria-label="淘汰"
              className="absolute inset-0 z-10 size-full scale-125 text-red drop-shadow-stage-cross"
            >
              <path
                d="M4 4 L20 20 M20 4 L4 20"
                stroke="currentColor"
                strokeWidth="4"
                strokeLinecap="round"
                fill="none"
              />
            </svg>
          )}
        </div>
        <RoleBadge
          label={roleLabel}
          tone={roleTone}
          className={cn(
            'drop-shadow-stage-soft transition-[opacity,filter] duration-300',
            stateClass,
          )}
        />
      </div>
    </div>
  );
}
