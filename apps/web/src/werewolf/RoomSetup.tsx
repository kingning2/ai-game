import { useEffect, useMemo, useState } from 'react';
import { cn } from '@agent-packages/shared-ui';
import { AGENT_POOL, dealGame, type BoardDef, type DealtGame } from './boards';
import { HostPanel } from './HostPanel';
import { MOCK_HOST } from './mock';

/** 角色阵营 → 展示色（狼红 / 神蓝 / 民灰），与舞台 roleTone 语义一致 */
const ROLE_CHIP: Record<string, string> = {
  werewolf: 'bg-red/15 text-red ring-red/40',
  seer: 'bg-blue/15 text-blue ring-blue/40',
  witch: 'bg-blue/15 text-blue ring-blue/40',
  guard: 'bg-blue/15 text-blue ring-blue/40',
  hunter: 'bg-blue/15 text-blue ring-blue/40',
  villager: 'bg-white/10 text-white/70 ring-white/20',
};

const ROLE_LABEL: Record<string, string> = {
  werewolf: '狼人',
  seer: '预言家',
  witch: '女巫',
  guard: '守卫',
  hunter: '猎人',
  villager: '平民',
};

/** 按 board 生成夜间流程描述（与 FlowDemo 剧本环节一一对应） */
function nightFlow(board: BoardDef): string {
  const steps = ['狼人睁眼，统一意见后刀人', '预言家睁眼，查验一名玩家'];
  if (board.guardSeat !== undefined) steps.push('守卫睁眼，守护一人');
  if (board.witchSeat !== undefined) steps.push('女巫睁眼，决定是否用药');
  return steps.join(' → ');
}

/** 开场规则条目：先讲规则再发身份，不可跳过 */
function buildRuleLines(board: BoardDef, roomName: string): string[] {
  return [
    `欢迎来到「${roomName}」。本局 ${board.total} 名 Agent 参战，阵容：${board.desc}。`,
    `夜晚流程：${nightFlow(board)}。`,
    '白天流程：公布昨晚死讯 → 出局者留遗言 → 依次发言 → 全员投票放逐一人。',
    '胜负规则：狼人全部出局，好人阵营获胜；当狼人数量不少于好人时，狼人阵营获胜。',
    '所有身份由系统随机发放——每一局，都是全新的博弈。',
  ];
}

export type RoomSetupProps = {
  board: BoardDef;
  roomName: string;
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
export function RoomSetup({ board, roomName, onComplete, onBack }: RoomSetupProps) {
  const [step, setStep] = useState<'picking' | 'rules' | 'dealing'>('picking');
  const [picked, setPicked] = useState<number[]>([]);
  const [dealt, setDealt] = useState<DealtGame | null>(null);
  const [shuffling, setShuffling] = useState(true);
  const [rulesDone, setRulesDone] = useState(false);
  const [revealDone, setRevealDone] = useState(false);

  const ruleLines = useMemo(() => buildRuleLines(board, roomName), [board, roomName]);

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

  // 规则条目全部入场后才允许继续（不可跳过）
  useEffect(() => {
    if (step !== 'rules') return undefined;
    const timer = setTimeout(() => setRulesDone(true), ruleLines.length * 350 + 700);
    return () => clearTimeout(timer);
  }, [step, ruleLines.length]);

  // 发牌：先洗牌 1.6s，再逐座翻开（每张 280ms）
  useEffect(() => {
    if (step !== 'dealing') return undefined;
    const shuffleTimer = setTimeout(() => setShuffling(false), 1600);
    const revealTimer = setTimeout(() => setRevealDone(true), 1600 + board.total * 280 + 600);
    return () => {
      clearTimeout(shuffleTimer);
      clearTimeout(revealTimer);
    };
  }, [step, board.total]);

  const startDealing = () => {
    setDealt(
      dealGame(
        board,
        picked.map((i) => AGENT_POOL[i]),
      ),
    );
    setShuffling(true);
    setRevealDone(false);
    setStep('dealing');
  };

  return (
    <div className="h-dvh overflow-auto bg-black bg-gradient-to-b from-blue/50 via-black/60 to-black">
      <div className="mx-auto flex min-h-full max-w-3xl flex-col gap-6 px-6 py-8">
        {/* 顶栏：返回 + 步骤指示 */}
        <div className="flex items-center justify-between">
          {onBack ? (
            <button
              type="button"
              onClick={onBack}
              className="rounded-full bg-black/50 px-3 py-1.5 text-xs font-medium text-white/80 backdrop-blur-sm hover:bg-black/70"
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
                    ? 'bg-yellow/90 text-black ring-yellow'
                    : 'text-white/40 ring-white/15',
                )}
              >
                {i + 1 === 1 ? '① 选人' : i + 1 === 2 ? '② 规则' : '③ 发牌'}
              </span>
            ))}
          </div>
        </div>

        <h1 className="text-center text-2xl font-black tracking-wide text-white">
          {step === 'picking' && `选择参赛 Agent（${picked.length}/${board.total}）`}
          {step === 'rules' && '游戏规则'}
          {step === 'dealing' && '随机发牌'}
        </h1>

        {step === 'picking' && (
          <>
            <p className="text-center text-sm text-white/50">
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
                      'hover:-translate-y-0.5 hover:ring-white/40',
                      on ? 'bg-white/15 ring-yellow' : 'bg-white/5 ring-white/10',
                    )}
                  >
                    <img src={agent.avatarSrc} alt="" className="size-12 rounded-xl" />
                    <span className="w-full truncate text-center text-xs font-medium text-white">
                      {agent.name}
                    </span>
                    {on && (
                      <span className="animate-zoom-in absolute -top-1.5 -right-1.5 flex size-5 items-center justify-center rounded-full bg-yellow text-[10px] font-black text-black">
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
                className="rounded-full border border-white/20 px-4 py-2 text-xs text-white/70 hover:border-white/50 hover:text-white"
              >
                🎲 随机补齐
              </button>
              <button
                type="button"
                onClick={() => setPicked([])}
                className="rounded-full border border-white/20 px-4 py-2 text-xs text-white/70 hover:border-white/50 hover:text-white"
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
                    ? 'bg-yellow text-black hover:scale-[1.03] hover:bg-yellow'
                    : 'cursor-not-allowed bg-white/10 text-white/30',
                )}
              >
                确认选手，讲规则 →
              </button>
            </div>
          </>
        )}

        {step === 'rules' && (
          <>
            <HostPanel
              host={MOCK_HOST}
              message={`各位 Agent，我是本场主持人。发身份之前，先把规则讲清楚——请仔细听完。`}
              className="w-full"
            />
            <ol className="flex flex-col gap-3">
              {ruleLines.map((line, i) => (
                <li
                  key={line.slice(0, 12)}
                  style={{ animationDelay: `${i * 350}ms` }}
                  className="animate-rise flex gap-3 rounded-2xl bg-white/5 p-4 ring-1 ring-white/10"
                >
                  <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-yellow/90 text-xs font-black text-black">
                    {i + 1}
                  </span>
                  <p className="text-sm leading-relaxed text-white/85">{line}</p>
                </li>
              ))}
            </ol>
            {rulesDone && (
              <div className="animate-fade-in flex justify-center">
                <button
                  type="button"
                  onClick={startDealing}
                  className="rounded-full bg-yellow px-6 py-2.5 text-sm font-bold text-black shadow-card transition-transform duration-200 hover:scale-[1.03]"
                >
                  我已了解，开始发身份 →
                </button>
              </div>
            )}
          </>
        )}

        {step === 'dealing' && dealt && (
          <>
            <p className="text-center text-sm text-white/50">
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
                      ? 'animate-pulse bg-white/5 ring-white/10'
                      : 'animate-zoom-in bg-white/10 ring-white/20 [animation-timing-function:var(--ease-spring)]',
                  )}
                >
                  <img src={p.avatarSrc} alt="" className="size-12 rounded-xl" />
                  <span className="w-full truncate text-center text-xs font-medium text-white">
                    {p.id} 号 · {p.name}
                  </span>
                  {shuffling ? (
                    <span className="text-xs text-white/30">身份保密</span>
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
                  className="rounded-full bg-yellow px-6 py-2.5 text-sm font-bold text-black shadow-card transition-transform duration-200 hover:scale-[1.03]"
                >
                  🌙 天黑请闭眼，进入对局 →
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
