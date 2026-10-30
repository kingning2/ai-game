import { Avatar, Card } from '@agent-packages/shared-ui';
import { cn } from '@agent-packages/shared-ui';
import type { Player } from '../types';

export type SpeechPanelProps = {
  speaker: Player;
  text: string;
  /** 视觉变体：speech = 白天公开发言（蓝调）；wolf = 夜晚狼队耳语（红调） */
  variant?: 'speech' | 'wolf';
  className?: string;
};

/** 当前发言面板：观战视角实时看到发言 Agent 的内容；狼聊变体用红调区分夜谈氛围 */
export function SpeechPanel({ speaker, text, variant = 'speech', className }: SpeechPanelProps) {
  const isWolf = variant === 'wolf';
  return (
    <Card className={className}>
      <div className="flex items-center gap-2.5">
        <Avatar src={speaker.avatarSrc} fallback={speaker.name.slice(0, 1)} size="sm" />
        <span className="text-sm font-semibold text-label">{speaker.name}</span>
        <span className={cn('flex items-center gap-1 text-xs', isWolf ? 'text-red' : 'text-blue')}>
          <span
            className={cn('h-1.5 w-1.5 animate-pulse rounded-full', isWolf ? 'bg-red' : 'bg-blue')}
          />
          {isWolf ? '狼队耳语中' : '发言中'}
        </span>
      </div>
      <p
        className={cn(
          'mt-3 border-l-2 pl-3 text-sm leading-relaxed text-label',
          isWolf ? 'border-red/40' : 'border-blue/40',
        )}
      >
        {text}
      </p>
    </Card>
  );
}
