import { useEffect, useRef, useState } from 'react';
import { cn } from '@agent-packages/shared-ui';
import { AGENT_POOL, type BoardDef } from '../boards';
import { createRoom, startGame } from '../../apis/werewolf';
import type { Player } from '../types';
import { useRecorder } from '../arena/useRecorder';
import { AgentPicker } from './AgentPicker';
import { ScriptPanel } from './ScriptPanel';

export type RoomSetupProps = {
  board: BoardDef;
  /** 规则讲完进对局页：带入选名册（座位草稿，身份由 server 发牌在对局页揭示）；
   *  发牌/倒计时/亮牌仪式全部在对局页上演（roomCode 供 SSE 定位） */
  onComplete: (roomCode: string | undefined, roster: Player[]) => void;
};

type SetupStep = 'picking' | 'rules';

const STEPS: readonly { key: SetupStep; label: string }[] = [
  { key: 'picking', label: '① 选人' },
  { key: 'rules', label: '② 规则' },
];

/**
 * 房间开场三步向导编排器（创建房间不可直接跳进对局）：
 * ① 挑选参赛 Agent → ② 剧本面板讲规则（同时后端已预热开局）→ ③ 洗牌发身份 → 对局。
 * 步骤状态机与全部业务时序在这里；三个子组件均为受控纯展示。
 */
export function RoomSetup({ board, onComplete }: RoomSetupProps) {
  const [step, setStep] = useState<SetupStep>('picking');
  const [picked, setPicked] = useState<number[]>([]);
  /** 真实房间号（SQLite 落库成功后回填；规则页/发牌页展示） */
  const [roomCode, setRoomCode] = useState<string | undefined>(undefined);
  const roomCodeRef = useRef<string | undefined>(undefined);
  useEffect(() => {
    roomCodeRef.current = roomCode;
  }, [roomCode]);
  /** 开局启动守卫（StrictMode 双执行去重） */
  const launchRef = useRef(false);

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

  // 规则页：真实创建房间（SQLite 落库）+ 预热开局（幂等）：讲规则的同时 server
  // 已在夜里推进，规则讲完牌局已在 buffer。房间创建失败不阻断对局（仅无落库）。
  // 录制同步启动：介绍页开场即录，跳对局页后推流会话跨页面接管不中断（模块级单例）。
  const recorder = useRecorder();
  const { state: recState, recording: recBusy, start: recStart } = recorder;
  useEffect(() => {
    if (step === 'rules' && recState === 'idle' && !recBusy) {
      void recStart(`${board.name}-${Date.now()}`);
    }
  }, [step, board.name, recState, recBusy, recStart]);
  useEffect(() => {
    if (step !== 'rules') return undefined;
    // StrictMode 双执行守卫：创建房间 + 开局只发一次（board.id 即 server 剧本名，同名直用）
    if (!launchRef.current) {
      launchRef.current = true;
      const boardId = board.id;
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
    return undefined;
  }, [step, board.id, picked]);

  const launchArena = () => {
    const roster: Player[] = picked.map((i, idx) => ({
      id: idx + 1,
      name: AGENT_POOL[i].name,
      avatarSrc: AGENT_POOL[i].avatarSrc,
      role: board.roles[idx],
      status: 'alive' as const,
    }));
    onComplete(roomCodeRef.current, roster);
  };

  return (
    <div className="flex h-dvh flex-col overflow-hidden bg-arena bg-gradient-to-b from-arena-glow/50 via-arena to-arena">
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto flex min-h-full max-w-3xl flex-col gap-6 px-6 py-8">
          {/* 顶栏：步骤指示（返回导航由 AppLayout 壳提供） */}
          <div className="flex items-center justify-between">
            <span />
            <div className="flex items-center gap-2 text-xs">
              {STEPS.map(({ key, label }) => (
                <span
                  key={key}
                  className={cn(
                    'rounded-full px-2.5 py-1 font-medium ring-1 transition-colors duration-200',
                    step === key
                      ? 'bg-arena-gold/15 text-arena ring-arena-gold'
                      : 'text-arena-muted ring-arena-line',
                  )}
                >
                  {label}
                </span>
              ))}
            </div>
          </div>

          <h1 className="text-center text-2xl font-black tracking-wide text-arena-text">
            {step === 'picking' && `选择参赛 Agent（${picked.length}/${board.total}）`}
            {step === 'rules' && '游戏规则'}
          </h1>

          {roomCode && step !== 'picking' && (
            <p className="text-center text-xs tracking-widest text-arena-muted">
              房间号 <span className="font-mono font-bold text-arena-gold">{roomCode}</span> ·
              已存档
            </p>
          )}

          {step === 'picking' && (
            <AgentPicker
              board={board}
              picked={picked}
              onToggle={toggle}
              onAutoFill={autoFill}
              onClear={() => setPicked([])}
              onConfirm={() => setStep('rules')}
            />
          )}

          {step === 'rules' && <ScriptPanel board={board} onLaunch={launchArena} />}
        </div>
      </div>
    </div>
  );
}
