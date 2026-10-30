import { Avatar, cn } from '@agent-packages/shared-ui';
import type { Host } from '../types';

export type HostPanelProps = {
  host: Host;
  /** 当前主持词：流程指令（天黑请闭眼 / 请发言 / 请投票…） */
  message: string;
  className?: string;
};

/** 主持人面板：系统流程声部（紫色调），置于玩家发言面板上方，负责发令与宣布 */
export function HostPanel({ host, message, className }: HostPanelProps) {
  return (
    <div className={cn('rounded-xl bg-arena/50 p-4 shadow-card backdrop-blur-sm', className)}>
      <div className="flex items-center gap-2.5">
        <Avatar
          fallback={host.avatarFallback ?? host.name.slice(0, 1)}
          size="sm"
          className="bg-purple/15 text-purple"
        />
        <span className="text-sm font-semibold text-arena-text">{host.name}</span>
        <span className="rounded-full bg-purple/15 px-2 py-0.5 text-xs font-medium text-purple">
          主持中
        </span>
      </div>
      <p className="mt-3 border-l-2 border-purple/60 pl-3 text-sm leading-relaxed text-arena-text">
        {message}
      </p>
    </div>
  );
}
