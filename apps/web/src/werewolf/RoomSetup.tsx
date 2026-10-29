import { useEffect, useMemo, useRef, useState } from 'react';
import { cn } from '@agent-packages/shared-ui';
import { AGENT_POOL, dealtFromServer, dealGame, type BoardDef, type DealtGame } from './boards';
import { CaptionBar } from './CaptionBar';
import { createRoom, fetchGameStatus, startGame, toServerBoard } from './liveClient';

/** 角色阵营 → 展示色（狼红 / 神蓝 / 民灰），与舞台 roleTone 语义一致 */
const ROLE_CHIP: Record<string, string> = {
  werewolf: 'bg-red/15 text-red ring-red/40',
  seer: 'bg-blue/15 text-blue ring-blue/40',
  witch: 'bg-blue/15 text-blue ring-blue/40',
  guard: 'bg-blue/15 text-blue ring-blue/40',
  hunter: 'bg-blue/15 text-blue ring-blue/40',
  villager: 'bg-arena-raised text-arena-muted ring-arena-line',
};

const ROLE_LABEL: Record<string, string> = {
  werewolf: '狼人',
  seer: '预言家',
  witch: '女巫',
  guard: '守卫',
  hunter: '猎人',
  villager: '平民',
};

/** 按 board 生成夜间流程描述（快闪阶段副文案，与引擎夜行动顺序一致） */
function nightFlow(board: BoardDef): string {
  const steps = ['狼人刀人', '预言家验人'];
  if (board.guardSeat !== undefined) steps.push('守卫守护');
  if (board.witchSeat !== undefined) steps.push('女巫用药');
  return steps.join(' → ');
}

/**
 * 身份卡展示序列（短视频讲解）：图标 + 一句职能口诀。
 * 狼人口诀带 ×N 计数；守卫/女巫按板子配置过滤。
 */
const ROLE_CARDS: { role: string; icon: string; tagline: string }[] = [
  { role: 'werewolf', icon: '🐺', tagline: '每晚睁眼，与狼伴共谋一个猎物' },
  { role: 'seer', icon: '🔮', tagline: '每晚查验一人：好人，还是狼' },
  { role: 'witch', icon: '⚗️', tagline: '一瓶解药一瓶毒，整局各一次' },
  { role: 'guard', icon: '🛡️', tagline: '每晚守护一人，挡下狼刀' },
  { role: 'hunter', icon: '🏹', tagline: '出局时开枪，带走一名玩家' },
  { role: 'villager', icon: '👤', tagline: '没有技能，靠发言与推理找出狼' },
];

/** 卡片底色（狼红/神蓝/民灰），与舞台 roleTone 语义一致 */
const CARD_TONE: Record<string, string> = {
  werewolf: 'from-red/30 to-red/10 ring-red/50',
  seer: 'from-blue/30 to-blue/10 ring-blue/50',
  witch: 'from-blue/30 to-blue/10 ring-blue/50',
  guard: 'from-blue/30 to-blue/10 ring-blue/50',
  hunter: 'from-blue/30 to-blue/10 ring-blue/50',
  villager: 'from-arena-raised to-transparent ring-arena-line',
};

const CARD_MS = 1600; // 每张身份卡停留时长
export type RoomSetupProps = {
  board: BoardDef;
  /** 发牌完成、玩家确认后进入对局 */
  onComplete: (dealt: DealtGame) => void;
  onBack?: () => void;
};

/**
 * 房间开场三步向导（创建房间不可直接跳进对局）：
 * ① 挑选参赛 Agent（恰好 board.total 人）→ ② 主持人讲规则（条目播完才能继续）
 * → ③ 洗牌 + 随机发身份（逐座翻牌动画）→ 进入对局。
 * 与大厅同视觉语言：夜幕黑底 + 蓝渐变 + 白字。
 */
export function RoomSetup({ board, onComplete, onBack }: RoomSetupProps) {
  const [step, setStep] = useState<'picking' | 'rules' | 'dealing'>('picking');
  const [picked, setPicked] = useState<number[]>([]);
  const [dealt, setDealt] = useState<DealtGame | null>(null);
  const [shuffling, setShuffling] = useState(true);
  // 规则页短视频时间轴：钩子(1.4s) → 身份卡逐张(CARD_MS/张) → 流程快闪(2.2s) → CTA
  const [rulePhase, setRulePhase] = useState<'hook' | 'cards' | 'flow' | 'done'>('hook');
  const [cardIdx, setCardIdx] = useState(0);
  const [revealDone, setRevealDone] = useState(false);
  /** 真实房间号（SQLite 落库成功后回填；规则页/发牌页展示） */
  const [roomCode, setRoomCode] = useState<string | undefined>(undefined);
  const roomCodeRef = useRef<string | undefined>(undefined);
  useEffect(() => {
    roomCodeRef.current = roomCode;
  }, [roomCode]);
  /** 开局启动守卫（StrictMode 双执行去重） */
  const launchRef = useRef(false);

  const visibleCards = useMemo(
    () =>
      ROLE_CARDS.filter((c) => c.role !== 'guard' || board.guardSeat !== undefined).filter(
        (c) => c.role !== 'witch' || board.witchSeat !== undefined,
      ),
    [board.guardSeat, board.witchSeat],
  );
  const wolfCount = board.roles.filter((r) => r === 'werewolf').length;

  const toggle = (idx: number) => {
    setPicked((cur) =>
      cur.includes(idx)
        ? cur.filter((i) => i !== idx)
        : cur.length >= board.total
          ? cur
          : [...cur, idx],
    );
  };

  const autoFill = () => {
    const rest = AGENT_POOL.map((_, i) => i).filter((i) => !picked.includes(i));
    const need = board.total - picked.length;
    const shuffled = rest.toSorted(() => Math.random() - 0.5).slice(0, need);
    setPicked((cur) => [...cur, ...shuffled]);
  };

  // 规则页时间轴推进：钩子→卡轮播→快闪→CTA；同时真实创建房间（SQLite 落库）
  // + 预热开局（幂等）：讲规则的同时 server 已在夜里推进，规则讲完牌局已在 buffer。
  // 房间创建失败不阻断对局（仅无落库）。
  useEffect(() => {
    if (step !== 'rules') return undefined;
    const boardId = toServerBoard(board.id);
    // StrictMode 双执行守卫：创建房间 + 开局只发一次
    if (boardId && !launchRef.current) {
      launchRef.current = true;
      const names = picked.map((i) => AGENT_POOL[i].name);
      const launch = async (): Promise<void> => {
        try {
          const room = await createRoom(boardId, names);
          setRoomCode(room.code);
          await startGame(boardId, names, room.code);
        } catch {
          // 房间服务不可用 / 已有对局：降级为无房间开局（幂等 409 容忍）
          await startGame(boardId, names).catch(() => undefined);
        }
      };
      void launch();
    }
    if (rulePhase === 'hook') {
      const t = setTimeout(() => setRulePhase('cards'), 1400);
      return () => clearTimeout(t);
    }
    if (rulePhase === 'cards') {
      if (cardIdx >= visibleCards.length - 1) {
        const t = setTimeout(() => setRulePhase('flow'), CARD_MS);
        return () => clearTimeout(t);
      }
      const t = setTimeout(() => setCardIdx((i) => i + 1), CARD_MS);
      return () => clearTimeout(t);
    }
    if (rulePhase === 'flow') {
      const t = setTimeout(() => setRulePhase('done'), 2200);
      return () => clearTimeout(t);
    }
    return undefined;
  }, [step, rulePhase, cardIdx, visibleCards.length, board.id, picked]);

  const skipRules = () => {
    setRulePhase('done');
    setCardIdx(visibleCards.length - 1);
  };

  // 发牌：先洗牌 1.6s，再逐座翻开（每张 280ms）；同时等 server 真实发牌结果
  useEffect(() => {
    if (step !== 'dealing') return undefined;
    const shuffleTimer = setTimeout(() => setShuffling(false), 1600);
    const revealTimer = setTimeout(() => setRevealDone(true), 1600 + board.total * 280 + 600);
    // 采用 server 引擎的真实发牌（座位+身份）；拉不到时回退本地随机（离线剧本模式）。
    // 轮询用 setTimeout 链（必须顺序等待 server 预热开局，不能并行拉）
    let cancelled = false;
    const localFallback = () => {
      if (!cancelled)
        setDealt(
          dealGame(
            board,
            picked.map((i) => AGENT_POOL[i]),
          ),
        );
    };
    const tryAdopt = (attempt: number): void => {
      if (cancelled) return;
      void (async () => {
        try {
          // 带 ?room= 定位本房对局：不能拿全局 latest（可能是别的房间的发牌）
          const roomCode = roomCodeRef.current;
          const status = await fetchGameStatus(roomCode ? { room: roomCode } : undefined);
          if (
            (status.status === 'running' || status.status === 'paused') &&
            status.seats.length === board.total &&
            (status.seat_roles?.length ?? 0) === board.total
          ) {
            if (!cancelled) {
              setDealt(
                dealtFromServer(board, status.seats, status.seat_roles!, roomCodeRef.current),
              );
            }
            return; // 采纳成功
          }
        } catch {
          localFallback(); // server 不可达：本地随机发牌（离线剧本）
          return;
        }
        if (attempt < 9) {
          setTimeout(() => tryAdopt(attempt + 1), 1000);
        } else {
          localFallback();
        }
      })();
    };
    setTimeout(() => tryAdopt(0), 1000);
    return () => {
      cancelled = true;
      clearTimeout(shuffleTimer);
      clearTimeout(revealTimer);
    };
  }, [step, board, picked]);

  const startDealing = () => {
    setDealt(null);
    setShuffling(true);
    setRevealDone(false);
    setStep('dealing');
  };

  return (
    <div className="flex h-dvh flex-col overflow-hidden bg-arena bg-gradient-to-b from-arena-glow/50 via-arena to-arena">
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto flex min-h-full max-w-3xl flex-col gap-6 px-6 py-8">
          {/* 顶栏：返回 + 步骤指示 */}
          <div className="flex items-center justify-between">
            {onBack ? (
              <button
                type="button"
                onClick={onBack}
                className="rounded-full bg-arena/50 px-3 py-1.5 text-xs font-medium text-arena-muted backdrop-blur-sm hover:bg-arena/70"
              >
                ← 返回大厅
              </button>
            ) : (
              <span />
            )}
            <div className="flex items-center gap-2 text-xs">
              {(['picking', 'rules', 'dealing'] as const).map((s, i) => (
                <span
                  key={s}
                  className={cn(
                    'rounded-full px-2.5 py-1 font-medium ring-1 transition-colors duration-200',
                    step === s
                      ? 'bg-arena-gold/15 text-arena ring-arena-gold'
                      : 'text-arena-muted ring-arena-line',
                  )}
                >
                  {i + 1 === 1 ? '① 选人' : i + 1 === 2 ? '② 规则' : '③ 发牌'}
                </span>
              ))}
            </div>
          </div>

          <h1 className="text-center text-2xl font-black tracking-wide text-arena-text">
            {step === 'picking' && `选择参赛 Agent（${picked.length}/${board.total}）`}
            {step === 'rules' && '游戏规则'}
            {step === 'dealing' && '随机发牌'}
          </h1>

          {roomCode && step !== 'picking' && (
            <p className="text-center text-xs tracking-widest text-arena-muted">
              房间号 <span className="font-mono font-bold text-arena-gold">{roomCode}</span> ·
              已存档
            </p>
          )}

          {step === 'picking' && (
            <>
              <p className="text-center text-sm text-arena-muted">
                {board.name} · {board.desc}——从 12 名 AI 中挑出本局 {board.total} 名选手
              </p>
              <div className="grid grid-cols-3 gap-3 sm:grid-cols-4">
                {AGENT_POOL.map((agent, idx) => {
                  const on = picked.includes(idx);
                  const order = picked.indexOf(idx);
                  return (
                    <button
                      key={agent.name}
                      type="button"
                      onClick={() => toggle(idx)}
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
                  onClick={autoFill}
                  className="rounded-full border border-arena-line px-4 py-2 text-xs text-arena-muted hover:border-arena-line hover:text-arena-text"
                >
                  🎲 随机补齐
                </button>
                <button
                  type="button"
                  onClick={() => setPicked([])}
                  className="rounded-full border border-arena-line px-4 py-2 text-xs text-arena-muted hover:border-arena-line hover:text-arena-text"
                >
                  清空
                </button>
                <button
                  type="button"
                  disabled={picked.length !== board.total}
                  onClick={() => setStep('rules')}
                  className={cn(
                    'rounded-full px-5 py-2 text-sm font-bold shadow-card transition-all duration-200',
                    picked.length === board.total
                      ? 'bg-arena-gold text-arena hover:scale-[1.03] hover:bg-arena-gold'
                      : 'cursor-not-allowed bg-arena-raised text-arena-muted',
                  )}
                >
                  确认选手，讲规则 →
                </button>
              </div>
            </>
          )}

          {step === 'rules' && (
            <div className="flex flex-col items-center gap-6">
              {/* 短视频时间轴：钩子 → 身份卡轮播 → 流程快闪 → CTA，全程可跳过 */}
              {rulePhase !== 'done' && (
                <button
                  type="button"
                  onClick={skipRules}
                  className="animate-fade-in self-end rounded-full px-3 py-1 text-xs text-arena-muted hover:text-arena-text"
                >
                  跳过讲解 →
                </button>
              )}

              {rulePhase === 'hook' && (
                <div className="flex min-h-[380px] flex-col items-center justify-center gap-4 text-center">
                  <p className="animate-zoom-in text-5xl font-black tracking-widest text-arena-text [animation-timing-function:var(--ease-spring)] drop-shadow-lg">
                    天黑请闭眼
                  </p>
                  <p className="animate-fade-in-up text-sm font-medium text-arena-muted [animation-delay:400ms]">
                    {board.total} 位 AI · {wolfCount} 匹狼潜伏 · 一夜定生死
                  </p>
                </div>
              )}

              {rulePhase === 'cards' && (
                <div className="flex min-h-[380px] w-full flex-col items-center justify-center gap-5">
                  {(() => {
                    const card = visibleCards[cardIdx];
                    return (
                      <div
                        key={card.role}
                        className={cn(
                          'animate-slide-in-right flex w-64 flex-col items-center gap-3 rounded-3xl bg-gradient-to-b p-7 ring-1 shadow-card [animation-timing-function:var(--ease-drawer)]',
                          CARD_TONE[card.role],
                        )}
                      >
                        <span className="animate-zoom-in text-6xl">{card.icon}</span>
                        <span className="animate-fade-in-up text-xl font-black text-arena-text [animation-delay:100ms]">
                          {ROLE_LABEL[card.role]}
                          {card.role === 'werewolf' && ` ×${wolfCount}`}
                        </span>
                        <span className="animate-fade-in-up text-center text-sm leading-relaxed text-arena-muted [animation-delay:220ms]">
                          {card.tagline}
                        </span>
                      </div>
                    );
                  })()}
                  <div className="flex gap-1.5">
                    {visibleCards.map((c, i) => (
                      <span
                        key={c.role}
                        className={cn(
                          'size-1.5 rounded-full transition-colors duration-200',
                          i === cardIdx ? 'bg-arena-gold' : 'bg-arena-raised',
                        )}
                      />
                    ))}
                  </div>
                </div>
              )}

              {(rulePhase === 'flow' || rulePhase === 'done') && (
                <div className="flex min-h-[380px] flex-col items-center justify-center gap-5">
                  <div className="flex items-center gap-2">
                    {[
                      { icon: '🌙', label: '天黑行动' },
                      { icon: '💬', label: '白天发言' },
                      { icon: '🗳️', label: '投票放逐' },
                    ].map((s, i) => (
                      <div
                        key={s.label}
                        style={{ animationDelay: `${i * 180}ms` }}
                        className="animate-zoom-in flex items-center gap-2 rounded-full bg-arena-raised px-4 py-2 ring-1 ring-arena-line [animation-timing-function:var(--ease-spring)]"
                      >
                        <span className="text-lg">{s.icon}</span>
                        <span className="text-sm font-bold text-arena-text">{s.label}</span>
                        {i < 2 && <span className="ml-1 text-arena-muted">→</span>}
                      </div>
                    ))}
                  </div>
                  <p className="animate-fade-in-up text-center text-xs leading-relaxed text-arena-muted [animation-delay:600ms]">
                    阵容：{board.desc}
                    <br />
                    夜晚：{nightFlow(board)} · 狼数≥好人即狼胜，狼全出局则好人胜
                  </p>
                </div>
              )}

              {rulePhase === 'done' && (
                <div className="animate-fade-in-up flex flex-col items-center gap-4">
                  <button
                    type="button"
                    onClick={startDealing}
                    className="rounded-full bg-arena-gold px-6 py-2.5 text-sm font-bold text-arena shadow-card transition-transform duration-200 hover:scale-[1.03]"
                  >
                    我已了解，开始发身份 →
                  </button>
                </div>
              )}

              <CaptionBar
                voice="主持人"
                text={
                  rulePhase === 'hook'
                    ? '各位 Agent，我是本场主持人。天黑请闭眼——先看看你会拿到什么牌。'
                    : rulePhase === 'cards'
                      ? `${ROLE_LABEL[visibleCards[cardIdx].role]}：${visibleCards[cardIdx].tagline}。`
                      : `夜晚${nightFlow(board)}；白天依次发言，全员投票放逐一人。狼全出局好人胜，狼数≥好人狼胜。`
                }
              />
            </div>
          )}

          {step === 'dealing' && dealt && (
            <>
              <p className="text-center text-sm text-arena-muted">
                {shuffling ? '身份牌洗牌中，请稍候……' : '身份已随机发放，点击座位查看身份'}
              </p>
              <div className="grid grid-cols-3 gap-3 sm:grid-cols-4">
                {dealt.players.map((p, i) => (
                  <div
                    key={p.id}
                    style={{ animationDelay: shuffling ? `${i * 120}ms` : `${i * 280}ms` }}
                    className={cn(
                      'flex flex-col items-center gap-2 rounded-2xl p-3 ring-1',
                      shuffling
                        ? 'animate-pulse bg-arena-raised ring-arena-line'
                        : 'animate-zoom-in bg-arena-raised ring-arena-line [animation-timing-function:var(--ease-spring)]',
                    )}
                  >
                    <img src={p.avatarSrc} alt="" className="size-12 rounded-xl" />
                    <span className="w-full truncate text-center text-xs font-medium text-arena-text">
                      {p.id} 号 · {p.name}
                    </span>
                    {shuffling ? (
                      <span className="text-xs text-arena-muted">身份保密</span>
                    ) : (
                      <span
                        className={cn(
                          'rounded-full px-2.5 py-0.5 text-xs font-bold ring-1',
                          ROLE_CHIP[p.role],
                        )}
                      >
                        {ROLE_LABEL[p.role]}
                      </span>
                    )}
                  </div>
                ))}
              </div>
              {revealDone && (
                <div className="animate-fade-in flex justify-center">
                  <button
                    type="button"
                    onClick={() => onComplete(dealt)}
                    className="rounded-full bg-arena-gold px-6 py-2.5 text-sm font-bold text-arena shadow-card transition-transform duration-200 hover:scale-[1.03]"
                  >
                    🌙 天黑请闭眼，进入对局 →
                  </button>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
