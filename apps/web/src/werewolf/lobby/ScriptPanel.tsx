import { useMemo } from 'react';
import { cn } from '@agent-packages/shared-ui';
import type { BoardDef } from '../boards';
import { CaptionBar } from '../CaptionBar';
import { campRows, ruleLinesOf } from './roles';

export type ScriptPanelProps = {
  board: BoardDef;
  onLaunch: () => void;
};

/**
 * 第②步：剧本讲解面板（暗色氛围 + 居中半透明卡片）——
 * 金色标题、阵营配色行、规则列表、底部字幕，主持人同步播报。
 * 纯展示：房间创建/开局/录制由 RoomSetup 编排，这里只讲剧本。
 */
export function ScriptPanel({ board, onLaunch }: ScriptPanelProps) {
  const camps = useMemo(() => campRows(board), [board]);
  const ruleLines = useMemo(() => ruleLinesOf(board), [board]);
  return (
    <div className="relative flex min-h-[560px] flex-col items-center justify-center gap-8">
      {/* 氛围层：烛光暖晕 + 暗角（令牌化，无硬编码色值） */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 rounded-3xl bg-[radial-gradient(ellipse_at_50%_18%,var(--color-arena-gold)_0%,transparent_55%)] opacity-10"
      />

      <div className="animate-fade-in-up relative w-full max-w-xl rounded-2xl bg-arena/80 p-8 ring-1 ring-arena-line shadow-card backdrop-blur-md">
        <h2 className="animate-zoom-in text-center text-3xl font-black tracking-widest text-arena-gold [animation-timing-function:var(--ease-spring)]">
          {board.total}人{board.name}
        </h2>

        <div className="mt-6 flex flex-col gap-2.5 text-sm leading-relaxed">
          {camps.map((camp, i) => (
            <p
              key={camp.label}
              style={{ animationDelay: `${200 + i * 160}ms` }}
              className={cn(
                'animate-fade-in-up font-bold',
                camp.tone === 'god' && 'text-arena-gold',
                camp.tone === 'wolf' && 'text-red',
                camp.tone === 'folk' && 'text-arena-text',
              )}
            >
              <span className={camp.tone === 'folk' ? 'text-red' : undefined}>{camp.label}</span>
              <span className="font-normal text-arena-text">：{camp.text}</span>
            </p>
          ))}
        </div>

        <div className="mt-5 flex flex-col gap-2 border-t border-arena-line pt-5 text-sm leading-relaxed">
          {ruleLines.map((line, i) => (
            <p
              key={line}
              style={{ animationDelay: `${600 + i * 200}ms` }}
              className={cn(
                'animate-fade-in-up',
                i === 0 ? 'font-bold text-red' : 'text-arena-muted',
              )}
            >
              {line}
            </p>
          ))}
        </div>
      </div>

      <p className="animate-fade-in-up text-center text-base font-bold tracking-wide text-arena-text [animation-delay:900ms]">
        本局游戏为{board.name}
      </p>

      <div className="animate-fade-in-up flex flex-col items-center gap-3 [animation-delay:1200ms]">
        <button
          type="button"
          onClick={onLaunch}
          className="rounded-full bg-arena-gold px-6 py-2.5 text-sm font-bold text-arena shadow-card transition-transform duration-200 hover:scale-[1.03]"
        >
          我已了解，开始发身份 →
        </button>
      </div>

      <CaptionBar
        voice="主持人"
        // 介绍页主持人走 Edge 免费接口（女声），无需本地模型
        voices={{ 主持人: 'zh-CN-XiaoxiaoNeural' }}
        text={
          board.intro?.length
            ? board.intro.join(' ')
            : `本局游戏为${board.name}。${camps.map((c) => `${c.label}：${c.text}。`).join('')}${ruleLines.join(' ')}`
        }
      />
    </div>
  );
}
