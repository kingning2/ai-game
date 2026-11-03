import { cn } from '@agent-packages/shared-ui';
import { AGENT_POOL, type BoardDef } from '../boards';

export type AgentPickerProps = {
  board: BoardDef;
  /** 已选 Agent 下标（受控：状态由 RoomSetup 持有，开局需要名单） */
  picked: number[];
  onToggle: (idx: number) => void;
  onAutoFill: () => void;
  onClear: () => void;
  onConfirm: () => void;
};

/** 第①步：从 AI 池里挑本局参赛 Agent（恰好 board.total 人） */
export function AgentPicker({
  board,
  picked,
  onToggle,
  onAutoFill,
  onClear,
  onConfirm,
}: AgentPickerProps) {
  const ready = picked.length === board.total;
  return (
    <>
      <p className="text-center text-sm text-arena-muted">
        {board.name} · {board.desc}——从 {AGENT_POOL.length} 名 AI 中挑出本局 {board.total} 名选手
      </p>
      <div className="grid grid-cols-3 gap-3 sm:grid-cols-4">
        {AGENT_POOL.map((agent, idx) => {
          const on = picked.includes(idx);
          const order = picked.indexOf(idx);
          return (
            <button
              key={agent.name}
              type="button"
              onClick={() => onToggle(idx)}
              style={{ animationDelay: `${idx * 45}ms` }}
              className={cn(
                'animate-rise relative flex flex-col items-center gap-2 rounded-2xl p-3 ring-1 transition-all duration-200',
                'hover:-translate-y-0.5 hover:ring-arena-gold/50',
                on ? 'bg-arena-raised ring-arena-gold' : 'bg-arena-raised ring-arena-line',
              )}
            >
              <img src={agent.avatarSrc} alt="" className="size-12 rounded-xl" />
              <span className="w-full truncate text-center text-xs font-medium text-arena-text">
                {agent.name}
              </span>
              {on && (
                <span className="animate-zoom-in absolute -top-1.5 -right-1.5 flex size-5 items-center justify-center rounded-full bg-arena-gold text-[10px] font-black text-arena">
                  {order + 1}
                </span>
              )}
            </button>
          );
        })}
      </div>
      <div className="flex items-center justify-center gap-3">
        <button
          type="button"
          onClick={onAutoFill}
          className="rounded-full border border-arena-line px-4 py-2 text-xs text-arena-muted hover:border-arena-line hover:text-arena-text"
        >
          🎲 随机补齐
        </button>
        <button
          type="button"
          onClick={onClear}
          className="rounded-full border border-arena-line px-4 py-2 text-xs text-arena-muted hover:border-arena-line hover:text-arena-text"
        >
          清空
        </button>
        <button
          type="button"
          disabled={!ready}
          onClick={onConfirm}
          className={cn(
            'rounded-full px-5 py-2 text-sm font-bold shadow-card transition-all duration-200',
            ready
              ? 'bg-arena-gold text-arena hover:scale-[1.03] hover:bg-arena-gold'
              : 'cursor-not-allowed bg-arena-raised text-arena-muted',
          )}
        >
          确认选手，讲规则 →
        </button>
      </div>
    </>
  );
}
