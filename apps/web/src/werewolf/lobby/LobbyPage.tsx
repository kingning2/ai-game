import { useEffect, useState } from 'react';
import { Avatar, SegmentedControl, cn } from '@agent-packages/shared-ui';
import { useNavigate } from 'react-router';
import { AGENT_POOL, BOARDS, ROLE_CN, type BoardDef } from '../boards';
import { fetchLeaderboard, fetchRooms } from '../../apis/werewolf';
import type { LeaderboardRowDto, RoomInfo } from '../../apis/types';

function boardOf(id: string): BoardDef {
  return BOARDS.find((b) => b.id === id) ?? BOARDS[0];
}

/** server 板子 id → 本地板子（房间列表观战入口用） */
function boardFromServer(boardId: string): BoardDef {
  const localId =
    boardId === 'all_stars' ? 'allstars12' : boardId === 'nine_standard' ? 'standard9' : 'rookie6';
  return boardOf(localId);
}

/** 房间列表刷新间隔（进行中房间状态跟随 server） */
const ROOMS_REFRESH_MS = 15_000;

/** 头像映射：选手池按名字取图，找不到回退默认（末字缩写） */
function avatarOf(name: string): string | undefined {
  return AGENT_POOL.find((agent) => agent.name === name)?.avatarSrc;
}

/** 前三名徽标色：金银铜 */
const RANK_CLS: Record<number, string> = {
  1: 'bg-arena-gold/15 text-arena',
  2: 'bg-arena-raised text-arena',
  3: 'bg-orange/70 text-arena',
};

/**
 * 狼人杀房间大厅：选择玩法 + 真实房间列表（SQLite）。
 * 房间只分两类：进行中（可观战）与历史（已结束）；等待只是建房→开局的毫秒级过渡，不展示。
 */
export function LobbyPage() {
  const navigate = useNavigate();
  const [boardId, setBoardId] = useState('allstars12');
  /** 真实房间（SQLite）；server 不可达时置 null（隐藏列表区块） */
  const [rooms, setRooms] = useState<RoomInfo[] | null>(null);
  /** 胜利排行榜（跨对局聚合）；server 不可达或还没打完过局为空 */
  const [leaderboard, setLeaderboard] = useState<LeaderboardRowDto[]>([]);
  /** 底部 Tab：历史房间 / 战绩排行榜（点击才展示，排行榜切入才拉取） */
  const [tab, setTab] = useState<'history' | 'board'>('history');
  const board = boardOf(boardId);

  // 拉取真实房间（新→旧，server 先清扫孤儿房）；定时刷新跟随对局状态
  useEffect(() => {
    let cancelled = false;
    const load = (): void => {
      void fetchRooms()
        .then((list) => (cancelled ? undefined : setRooms(list)))
        .catch(() => (cancelled ? undefined : setRooms(null)));
    };
    load();
    const timer = setInterval(load, ROOMS_REFRESH_MS);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, []);

  // 排行榜：切到该 Tab 才拉取（每次切入刷新一次）
  useEffect(() => {
    if (tab !== 'board') return undefined;
    let cancelled = false;
    void fetchLeaderboard()
      .then((rows) => (cancelled ? undefined : setLeaderboard(rows)))
      .catch(() => (cancelled ? undefined : setLeaderboard([])));
    return () => {
      cancelled = true;
    };
  }, [tab]);

  const active = (rooms ?? []).filter((r) => r.status === 'playing');
  const history = (rooms ?? []).filter((r) => r.status === 'finished');

  return (
    <div className="flex h-dvh flex-col overflow-hidden bg-arena bg-gradient-to-b from-arena-glow/50 via-arena to-arena">
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto flex min-h-full max-w-5xl flex-col gap-8 px-6 py-10">
          <header className="text-center">
            <h1 className="text-4xl font-black tracking-widest text-arena-gold drop-shadow-lg">
              AI 狼人杀
            </h1>
            <p className="mt-2 flex items-center justify-center gap-3 text-sm text-arena-muted">
              对局大厅 · 选玩法，进房间，看 Agent 博弈
              <button
                type="button"
                onClick={() => void navigate('/werewolf/spectate')}
                className="text-arena-gold/80 underline-offset-2 hover:text-arena-gold hover:underline"
              >
                静态观战示例 →
              </button>
            </p>
          </header>

          {/* 玩法选择：板子决定人数；创建即走真实建房（选人向导 → SQLite 落库） */}
          <section>
            <div className="mb-3 flex items-center justify-between">
              <h2 className="text-sm font-bold tracking-wider text-arena-muted">选择玩法</h2>
              <button
                type="button"
                onClick={() => {
                  void navigate(`/werewolf/setup/${board.id}`);
                }}
                className="rounded-full bg-arena-gold/20 px-4 py-1.5 text-xs font-bold text-arena shadow-card transition-colors hover:bg-arena-gold"
              >
                + 创建 {board.total} 人房
              </button>
            </div>
            <div className="grid gap-3 sm:grid-cols-3">
              {BOARDS.map((b) => (
                <button
                  key={b.id}
                  type="button"
                  onClick={() => setBoardId(b.id)}
                  className={cn(
                    'rounded-xl border px-4 py-4 text-left transition-all duration-200 ease-spring',
                    b.id === boardId
                      ? 'border-arena-gold/70 bg-arena-gold/10 ring-2 ring-arena-gold/50'
                      : 'border-arena-line bg-arena-raised hover:border-arena-line',
                  )}
                >
                  <div className="flex items-baseline justify-between gap-2">
                    <span className="font-bold text-arena-text">{b.name}</span>
                    <span
                      className={cn(
                        'shrink-0 rounded-full px-2 py-0.5 text-xs font-bold',
                        b.id === boardId
                          ? 'bg-arena-gold/15 text-arena'
                          : 'bg-arena-raised text-arena-muted',
                      )}
                    >
                      {b.total} 人
                    </span>
                  </div>
                  <p className="mt-2 text-xs leading-relaxed text-arena-muted">{b.desc}</p>
                  <div className="mt-3 flex flex-wrap gap-1">
                    {[...new Set(b.roles)].map((r) => (
                      <span
                        key={r}
                        className={cn(
                          'rounded px-1.5 py-0.5 text-[10px] font-semibold',
                          r === 'werewolf'
                            ? 'bg-red/25 text-red'
                            : 'bg-arena-gold/10 text-arena-gold',
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

          {/* 进行中房间：真实对局，可一键观战 */}
          {active.length > 0 && (
            <section>
              <h2 className="mb-3 text-sm font-bold tracking-wider text-arena-muted">进行中</h2>
              <ul className="flex flex-col gap-2">
                {active.map((room) => (
                  <li
                    key={room.code}
                    className="flex items-center gap-3 rounded-xl border border-arena-line bg-arena-raised px-4 py-2.5 text-sm"
                  >
                    <span className="font-mono font-bold tracking-widest text-arena-gold">
                      {room.code}
                    </span>
                    <span className="text-arena-muted">{room.name}</span>
                    <span className="inline-flex items-center gap-1.5 rounded-full bg-red/20 px-2 py-0.5 text-xs font-semibold text-red">
                      <span className="size-1.5 animate-pulse rounded-full bg-red" />
                      对局中
                    </span>
                    <span className="text-xs text-arena-muted">{room.created_at}</span>
                    <button
                      type="button"
                      onClick={() => {
                    void navigate('/werewolf/play', {
                      state: { boardId: boardFromServer(room.board_id).id, roomCode: room.code },
                    });
                  }}
                      className="ml-auto shrink-0 rounded-full bg-arena-gold/20 px-4 py-1.5 text-xs font-bold text-arena shadow-card transition-colors hover:bg-arena-gold"
                    >
                      观战
                    </button>
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* 历史房间 × 战绩排行榜：同一个 Tab 区，点击才展开对应内容 */}
          <section className="flex flex-col gap-3">
            <SegmentedControl<'history' | 'board'>
              value={tab}
              onChange={setTab}
              options={[
                {
                  value: 'history',
                  label: `历史房间${history.length > 0 ? ` · ${history.length}` : ''}`,
                },
                { value: 'board', label: '战绩排行榜' },
              ]}
              className="w-fit self-center px-6"
            />
            {tab === 'history' &&
              (history.length === 0 ? (
                <p className="rounded-xl border border-arena-line bg-arena-raised px-4 py-3 text-sm text-arena-muted">
                  还没有已结束的对局——开一局，结束后房间存档会出现在这里。
                </p>
              ) : (
                <ul className="flex flex-col gap-2">
                  {history.map((room) => (
                    <li
                      key={room.code}
                      className="flex items-center gap-3 rounded-xl border border-arena-line bg-arena-raised px-4 py-2.5 text-sm"
                    >
                      <span className="font-mono font-bold tracking-widest text-arena-muted">
                        {room.code}
                      </span>
                      <span className="text-arena-muted">{room.name}</span>
                      <span className="rounded-full bg-arena-raised px-2 py-0.5 text-xs font-bold text-arena-muted">
                        已结束
                      </span>
                      <span className="text-xs text-arena-muted">{room.created_at}</span>
                      <button
                        type="button"
                        onClick={() => {
                    void navigate('/werewolf/play', {
                      state: { boardId: boardFromServer(room.board_id).id, roomCode: room.code },
                    });
                  }}
                        className="ml-auto shrink-0 rounded-full border border-arena-line px-4 py-1.5 text-xs font-bold text-arena-muted transition-colors hover:border-arena-line hover:text-arena-text"
                      >
                        查看结果
                      </button>
                    </li>
                  ))}
                </ul>
              ))}
            {tab === 'board' &&
              (leaderboard.length === 0 ? (
                <p className="rounded-xl border border-arena-line bg-arena-raised px-4 py-3 text-sm text-arena-muted">
                  还没有打完过对局——开一局，胜者与 MVP 将在这里留名。
                </p>
              ) : (
                <ul className="grid grid-cols-2 gap-3">
                  {leaderboard.map((row, i) => (
                    <li
                      key={row.name}
                      className={cn(
                        'flex items-center gap-3 rounded-xl border px-4 py-3 text-sm',
                        i < 3
                          ? 'border-arena-gold/30 bg-arena-gold/15'
                          : 'border-arena-line bg-arena-raised',
                      )}
                    >
                      <span
                        className={cn(
                          'flex size-6 shrink-0 items-center justify-center rounded-full text-xs font-black',
                          RANK_CLS[i + 1] ?? 'bg-arena-raised text-arena-muted',
                        )}
                      >
                        {i + 1}
                      </span>
                      <Avatar
                        src={avatarOf(row.name)}
                        alt={row.name}
                        fallback={row.name.slice(-1)}
                        size="sm"
                      />
                      <span className="min-w-0 flex-1">
                        <span className="block truncate font-bold text-arena-text">{row.name}</span>
                        <span className="block text-xs text-arena-muted">
                          <span className="font-bold text-arena-gold/90">{row.wins}</span> 胜 /{' '}
                          {row.games} 场 · {(row.win_rate * 100).toFixed(0)}%
                        </span>
                      </span>
                      <span className="shrink-0 rounded-full bg-arena-gold/10 px-2 py-0.5 text-xs font-bold text-arena-gold/90">
                        MVP × {row.mvp_count}
                      </span>
                    </li>
                  ))}
                </ul>
              ))}
          </section>
        </div>
      </div>
    </div>
  );
}
