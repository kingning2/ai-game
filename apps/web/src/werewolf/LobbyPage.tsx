import { useState } from 'react';
import { cn } from '@agent-packages/shared-ui';
import { BOARDS, ROLE_CN, type BoardDef } from './boards';

/** 大厅房间（demo 本地态；引擎接入后由服务端房间列表驱动） */
export type Room = {
  id: string;
  name: string;
  boardId: string;
  /** 等待中 / 对局中 */
  status: 'waiting' | 'playing';
  /** 当前人数（等待中房间展示进度） */
  joined: number;
  host: string;
};

const INITIAL_ROOMS: Room[] = [
  {
    id: 'r1',
    name: '全明星巅峰局',
    boardId: 'allstars12',
    status: 'playing',
    joined: 12,
    host: '豆包',
  },
  {
    id: 'r2',
    name: '标准 9 人上分场',
    boardId: 'standard9',
    status: 'playing',
    joined: 9,
    host: '元宝',
  },
  {
    id: 'r3',
    name: '新手村练习赛',
    boardId: 'rookie6',
    status: 'waiting',
    joined: 3,
    host: 'Kimi',
  },
];

function boardOf(id: string): BoardDef {
  return BOARDS.find((b) => b.id === id) ?? BOARDS[0];
}

/** 房间行：名称 + 板子徽标 + 状态 + 人数 + 进入 */
function RoomRow({ room, onEnter }: { room: Room; onEnter: (room: Room) => void }) {
  const board = boardOf(room.boardId);
  return (
    <li
      className={cn(
        'flex items-center gap-4 rounded-xl border border-white/10 bg-white/5 px-5 py-4',
        'backdrop-blur-sm transition-colors hover:bg-white/10',
      )}
    >
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="truncate text-base font-bold text-white">{room.name}</span>
          <span
            className={cn(
              'inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-xs font-semibold',
              room.status === 'playing' ? 'bg-red/20 text-red' : 'bg-green/20 text-green',
            )}
          >
            {/* 对局中呼吸红点 / 等待中绿点（只动 opacity） */}
            <span
              className={cn(
                'size-1.5 rounded-full',
                room.status === 'playing' ? 'bg-red animate-pulse' : 'bg-green',
              )}
            />
            {room.status === 'playing' ? '对局中' : '等待中'}
          </span>
        </div>
        <p className="mt-1 text-xs text-white/50">
          {board.name} · {board.desc} · 房主 {room.host}
        </p>
      </div>
      <span className="shrink-0 text-sm font-bold text-yellow">
        {room.joined}
        <span className="text-white/40">/{board.total}</span>
      </span>
      <button
        type="button"
        onClick={() => onEnter(room)}
        className="shrink-0 rounded-full bg-yellow/90 px-4 py-1.5 text-xs font-bold text-black shadow-card transition-colors hover:bg-yellow"
      >
        {room.status === 'playing' ? '观战' : '进入'}
      </button>
    </li>
  );
}

/** 狼人杀房间大厅：选择玩法（人数随板子走）+ 房间列表 + 创建房间；左上回游戏大厅，右上 ⚙️ 进全局 AI 设置 */
export function LobbyPage({
  onBack,
  onEnter,
  onSpectate,
  onSettings,
}: {
  onBack: () => void;
  onEnter: (room: Room, board: BoardDef) => void;
  onSpectate?: () => void;
  onSettings?: () => void;
}) {
  const [boardId, setBoardId] = useState('allstars12');
  const [rooms, setRooms] = useState<Room[]>(INITIAL_ROOMS);
  const board = boardOf(boardId);

  const createRoom = () => {
    const no = rooms.filter((r) => r.boardId === boardId).length + 1;
    const room: Room = {
      id: `r${Date.now()}`,
      name: `${board.name} · ${no} 号房`,
      boardId,
      status: 'waiting',
      joined: board.total,
      host: '我',
    };
    setRooms((rs) => [room, ...rs]);
    // demo 语义：创建即满员开演（引擎接入后改为等待真实玩家就位）
    onEnter(room, board);
  };

  // 大厅是夜幕沉浸页（与观战页一致）：黑底 + 蓝色氛围渐变，亮暗主题统一，白字才可读
  return (
    <div className="h-dvh overflow-auto bg-black bg-gradient-to-b from-blue/50 via-black/60 to-black">
      <div className="mx-auto flex min-h-full max-w-5xl flex-col gap-8 px-6 py-10">
        <button
          type="button"
          onClick={onBack}
          className={cn(
            'w-fit rounded-full border border-white/15 px-3 py-1.5 text-xs text-white/70',
            'transition-colors duration-200 hover:border-white/40 hover:text-white active:scale-[0.98]',
          )}
        >
          ‹ 返回游戏大厅
        </button>
        <header className="text-center">
          <h1 className="text-4xl font-black tracking-widest text-yellow drop-shadow-lg">
            AI 狼人杀
          </h1>
          <p className="mt-2 flex items-center justify-center gap-3 text-sm text-white/50">
            对局大厅 · 选玩法，进房间，看 Agent 博弈
            {onSettings && (
              <button
                type="button"
                onClick={onSettings}
                className="rounded-full border border-white/15 px-2.5 py-0.5 text-xs text-white/70 transition-colors hover:border-white/40 hover:text-white"
              >
                ⚙️ AI 设置
              </button>
            )}
            {onSpectate && (
              <button
                type="button"
                onClick={onSpectate}
                className="text-yellow/80 underline-offset-2 hover:text-yellow hover:underline"
              >
                静态观战示例 →
              </button>
            )}
          </p>
        </header>

        {/* 玩法选择：板子决定人数 */}
        <section>
          <h2 className="mb-3 text-sm font-bold tracking-wider text-white/70">选择玩法</h2>
          <div className="grid gap-3 sm:grid-cols-3">
            {BOARDS.map((b) => (
              <button
                key={b.id}
                type="button"
                onClick={() => setBoardId(b.id)}
                className={cn(
                  'rounded-xl border px-4 py-4 text-left transition-all duration-200 ease-spring',
                  b.id === boardId
                    ? 'border-yellow/70 bg-yellow/10 ring-2 ring-yellow/50'
                    : 'border-white/10 bg-white/5 hover:border-white/25',
                )}
              >
                <div className="flex items-baseline justify-between gap-2">
                  <span className="font-bold text-white">{b.name}</span>
                  <span
                    className={cn(
                      'shrink-0 rounded-full px-2 py-0.5 text-xs font-bold',
                      b.id === boardId ? 'bg-yellow/90 text-black' : 'bg-white/10 text-white/60',
                    )}
                  >
                    {b.total} 人
                  </span>
                </div>
                <p className="mt-2 text-xs leading-relaxed text-white/50">{b.desc}</p>
                <div className="mt-3 flex flex-wrap gap-1">
                  {[...new Set(b.roles)].map((r) => (
                    <span
                      key={r}
                      className={cn(
                        'rounded px-1.5 py-0.5 text-[10px] font-semibold',
                        r === 'werewolf' ? 'bg-red/25 text-red' : 'bg-yellow/15 text-yellow',
                      )}
                    >
                      {ROLE_CN[r]}
                    </span>
                  ))}
                </div>
              </button>
            ))}
          </div>
        </section>

        {/* 房间列表 */}
        <section className="flex min-h-0 flex-1 flex-col">
          <div className="mb-3 flex items-center justify-between">
            <h2 className="text-sm font-bold tracking-wider text-white/70">房间列表</h2>
            <button
              type="button"
              onClick={createRoom}
              className="rounded-full bg-yellow/90 px-4 py-1.5 text-xs font-bold text-black shadow-card transition-colors hover:bg-yellow"
            >
              + 创建 {board.total} 人房
            </button>
          </div>
          <ul className="flex flex-col gap-3">
            {rooms.map((room) => (
              <RoomRow key={room.id} room={room} onEnter={(r) => onEnter(r, boardOf(r.boardId))} />
            ))}
          </ul>
        </section>
      </div>
    </div>
  );
}
