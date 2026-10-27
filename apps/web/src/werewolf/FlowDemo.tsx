import { useEffect, useMemo, useRef, useState } from 'react';
import { VStack, WerewolfStage, cn, type WerewolfPlayer } from '@agent-packages/shared-ui';
import { HostPanel } from './HostPanel';
import { MOCK_HOST } from './mock';
import { SpeechPanel } from './SpeechPanel';
import { buildPlayers, type BoardDef } from './boards';
import { fetchGameStatus, startGame, toServerBoard, type GameStatusDto } from './liveClient';
import type { Player } from './types';

/** 夜间动作标记：刀 / 盾 / 救 / 查验结果（demo 展示件，色语义与 RoleBadge 对齐） */
function SeatMark({ kind }: { kind: 'kill' | 'guard' | 'good' | 'wolf' | 'save' }) {
  const meta = {
    kill: { text: '刀', cls: 'bg-red text-white' },
    guard: { text: '盾', cls: 'bg-blue text-white' },
    good: { text: '好', cls: 'bg-green text-black' },
    wolf: { text: '狼', cls: 'bg-red text-white' },
    save: { text: '救', cls: 'bg-green text-black' },
  }[kind];
  return (
    <span
      className={cn(
        'flex size-7 items-center justify-center rounded-full text-xs font-black shadow-card',
        'ring-1 ring-white/30',
        meta.cls,
      )}
    >
      {meta.text}
    </span>
  );
}

/** 流程剧本步骤：每步声明该步的横幅/主持词/睁眼者/发言/标记/死亡/大字幕 */
type FlowStep = {
  phase: 'night' | 'day';
  phaseLabel: string;
  /** 主持人面板文案；缺省不显示面板 */
  host?: string;
  /** 当前发言者（狼队夜聊 / 遗言 / 白天发言） */
  speech?: { playerId: number; text: string };
  /** 夜晚睁眼座位（全彩 + 金色呼吸光环），其余存活座位深度压暗 */
  spotlightSeats?: number[];
  /** 座位动作标记（刀/盾/救/查验） */
  overlays?: { seat: number; kind: 'kill' | 'guard' | 'good' | 'wolf' | 'save' }[];
  /** 本步公布的死亡（座位打叉灰化） */
  deaths?: number[];
  caption?: string;
  duration: number;
};

const nameOf = (players: Player[], seat: number): string =>
  players.find((p) => p.id === seat)?.name ?? `${seat} 号`;

/** 按板子生成第 1 夜完整剧本：天黑 → 狼刀（讨论）→ 预言家查验 → [守卫] → [女巫] → 天亮结局 → 遗言 */
function buildScript(board: BoardDef, roomName: string, players: Player[]): FlowStep[] {
  const killName = nameOf(players, board.killTarget);
  const checkName = nameOf(players, board.checkTarget);
  const saved = board.witchSaves || board.guardSeat !== undefined;
  const steps: FlowStep[] = [
    {
      phase: 'night',
      phaseLabel: '🌙 天黑请闭眼',
      host: '夜幕降临，全体 Agent 请闭眼。',
      caption: `${roomName} · 第 1 夜 · 天黑请闭眼`,
      duration: 3000,
    },
    {
      phase: 'night',
      phaseLabel: '🐺 狼人行动',
      host: '狼人请睁眼，请统一今晚的意见。',
      spotlightSeats: board.wolfSeats,
      duration: 2800,
    },
    {
      phase: 'night',
      phaseLabel: '🐺 狼人行动',
      host: '狼人队内讨论中……',
      spotlightSeats: board.wolfSeats,
      speech: {
        playerId: board.wolfSeats[0],
        text: `白天 ${checkName} 的发言太干净了，先刀他，探探谁在保。`,
      },
      duration: 4200,
    },
    {
      phase: 'night',
      phaseLabel: '🐺 狼人行动',
      host: '狼人已统一意见，请闭眼。',
      spotlightSeats: board.wolfSeats,
      overlays: [{ seat: board.killTarget, kind: 'kill' }],
      caption: `狼刀已出，直指 ${board.killTarget} 号 ${killName}`,
      duration: 2800,
    },
    {
      phase: 'night',
      phaseLabel: '🔮 预言家查验',
      host: '预言家请睁眼，请查验一名玩家。',
      spotlightSeats: [board.seerSeat],
      duration: 2600,
    },
    {
      phase: 'night',
      phaseLabel: '🔮 预言家查验',
      host: `你查验了 ${board.checkTarget} 号 ${checkName}。`,
      spotlightSeats: [board.seerSeat],
      overlays: [{ seat: board.checkTarget, kind: 'good' }],
      caption: `${board.checkTarget} 号 ${checkName} · 好人`,
      duration: 3200,
    },
  ];

  if (board.guardSeat !== undefined) {
    steps.push(
      {
        phase: 'night',
        phaseLabel: '🛡️ 守卫守护',
        host: '守卫请睁眼，请选择今晚守护的人。',
        spotlightSeats: [board.guardSeat],
        duration: 2600,
      },
      {
        phase: 'night',
        phaseLabel: '🛡️ 守卫守护',
        host: '守卫做出了选择，请闭眼。',
        spotlightSeats: [board.guardSeat],
        overlays: [{ seat: board.killTarget, kind: 'guard' }],
        caption: `守卫的盾，挡在 ${board.killTarget} 号身前`,
        duration: 3200,
      },
    );
  }

  if (board.witchSeat !== undefined) {
    steps.push(
      {
        phase: 'night',
        phaseLabel: '⚗️ 女巫决策',
        host: `女巫请睁眼，今晚 ${board.killTarget} 号被袭击，是否使用解药？`,
        spotlightSeats: [board.witchSeat],
        duration: 3000,
      },
      board.witchSaves
        ? {
            phase: 'night',
            phaseLabel: '⚗️ 女巫决策',
            host: '女巫交出了解药，请闭眼。',
            spotlightSeats: [board.witchSeat],
            overlays: [{ seat: board.killTarget, kind: 'save' }],
            caption: `女巫的解药，浇在 ${killName} 身上`,
            duration: 3400,
          }
        : {
            phase: 'night',
            phaseLabel: '⚗️ 女巫决策',
            host: '女巫请闭眼。',
            spotlightSeats: [board.witchSeat],
            speech: {
              playerId: board.witchSeat,
              text: '解药第一晚就交出去了……今晚这刀，只能赌守卫了。',
            },
            duration: 4200,
          },
    );
  }

  steps.push(
    {
      phase: 'day',
      phaseLabel: '☀️ 天亮请睁眼',
      host: '天亮了，全体 Agent 请睁眼。',
      caption: saved
        ? '天亮了——昨晚是平安夜！'
        : `天亮了——${board.killTarget} 号 ${killName} 出局了`,
      ...(saved ? {} : { deaths: [board.killTarget] }),
      duration: 4000,
    },
    {
      phase: 'day',
      phaseLabel: '☀️ 白天发言',
      caption: saved ? '平安夜之后，好人阵营士气大振' : `${killName} 留下遗言，白天发言开始`,
      speech: {
        playerId: board.killTarget,
        text: saved
          ? '我居然还活着！昨晚保我的人，这一票我跟你走了。'
          : '我是好人，兄弟们替我报仇，别让狼队好过！',
      },
      duration: 4200,
    },
  );
  return steps;
}

/** 领域 Player → 舞台座位：夜晚非睁眼者深度压暗，睁眼/发言全彩，附带动作标记与死亡态 */
function toStagePlayer(p: Player, step: FlowStep): WerewolfPlayer {
  const spotlighted = step.spotlightSeats?.includes(p.id) ?? false;
  const speaking = step.speech?.playerId === p.id;
  const mark = step.overlays?.find((o) => o.seat === p.id);
  const dead = p.status === 'dead' || step.deaths?.includes(p.id) || false;
  const roleLabel: Record<Player['role'], string> = {
    werewolf: '狼人',
    seer: '预言家',
    witch: '女巫',
    guard: '守卫',
    hunter: '猎人',
    villager: '平民',
  };
  return {
    seat: p.id,
    name: p.name,
    avatarSrc: p.avatarSrc,
    roleLabel: roleLabel[p.role],
    roleTone: p.role === 'werewolf' ? 'wolf' : 'good',
    alive: !dead,
    speaking,
    spotlight: spotlighted && !speaking,
    dimmed: step.phase === 'night' && !spotlighted && !speaking && !dead,
    overlay: mark ? <SeatMark kind={mark.kind} /> : undefined,
  };
}

function liveToStageView(status: GameStatusDto, players: Player[], roomName: string) {
  const events = status.events;
  const last = events.at(-1);
  const dead = new Set(
    events
      .filter((e) => e.kind === 'died')
      .map((e) => e.seat)
      .filter((s) => s !== null),
  );
  // 发言/狼聊：最近一条带文本的事件（含说话座位）
  const talk = events
    .toReversed()
    .find((e) => (e.kind === 'speech' || e.kind === 'wolf_chat') && e.seat !== null);
  // 夜晚聚光：最近一条夜行动/狼聊/狼沉默的主角
  const spotlightSeat = events
    .toReversed()
    .find(
      (e) =>
        status.phase !== 'day' &&
        e.seat !== null &&
        (e.kind === 'night_action' || e.kind === 'wolf_chat' || e.kind === 'wolf_pass'),
    )?.seat;
  const speakingSeat = talk?.seat ?? null;
  const phase: 'night' | 'day' = status.phase === 'day' ? 'day' : 'night';
  const phaseLabel =
    status.status === 'ended'
      ? status.winner === 'werewolf'
        ? '🐺 狼人阵营获胜'
        : '☀️ 好人阵营获胜'
      : phase === 'night'
        ? `🌙 第 ${status.day} 夜`
        : `☀️ 第 ${status.day} 天`;
  const stagePlayers = players.map((p) => ({
    ...p,
    status: dead.has(p.id) ? ('dead' as const) : p.status,
  }));
  const speaker = speakingSeat !== null ? players.find((p) => p.id === speakingSeat) : undefined;
  return {
    events,
    last,
    phase,
    phaseLabel,
    caption: `${roomName} · 第 ${status.day} ${phase === 'night' ? '夜' : '天'} · 存活 ${status.alive_total}/${status.total}`,
    stagePlayers,
    speaker,
    speechText: talk?.text,
    spotlightSeats: spotlightSeat !== null && spotlightSeat !== undefined ? [spotlightSeat] : [],
    hostMessage: last?.text ?? '正在等待引擎推进……',
  };
}

/**
 * 房间对局演示：按板子生成玩家与剧本（或接收发牌结果），定时推进“天黑→狼刀→查验→[守卫]→[女巫]→天亮”，
 * 纯前端编排（引擎接入后由事件流驱动同一舞台）。播完停在末帧，可重播。
 */
export function FlowDemo({
  board,
  roomName,
  players: playersOverride,
  onBack,
}: {
  board: BoardDef;
  roomName: string;
  /** 发牌结果注入（RoomSetup 随机分配后传入）；缺省取选手池前 N 人 */
  players?: Player[];
  onBack?: () => void;
}) {
  const players = useMemo(() => playersOverride ?? buildPlayers(board), [board, playersOverride]);
  const [live, setLive] = useState<GameStatusDto | null>(null);
  const [liveError, setLiveError] = useState<string | null>(null);
  const startedRef = useRef(false);
  const serverBoard = toServerBoard(board.id);

  // live 模式：开局（幂等，已在对局则直接观战）+ 轮询事件流
  useEffect(() => {
    const target = toServerBoard(board.id);
    if (!target || startedRef.current) return undefined;
    startedRef.current = true;
    const seatNames = players.map((p) => p.name);
    let cancelled = false;
    const poll = async () => {
      try {
        const next = await fetchGameStatus();
        if (cancelled) return;
        if (next.status === 'idle') {
          setLive(await startGame(target, seatNames));
        } else {
          setLive(next);
        }
        setLiveError(null);
      } catch (err) {
        if (!cancelled) setLiveError(err instanceof Error ? err.message : '连接 server 失败');
      }
    };
    void poll();
    const timer = setInterval(poll, 1500);
    return () => {
      cancelled = true;
      clearInterval(timer);
      startedRef.current = false;
    };
  }, [board, players]);
  const script = useMemo(() => buildScript(board, roomName, players), [board, roomName, players]);
  const [step, setStep] = useState(0);
  const current = script[step];
  const finished = step === script.length - 1;

  // live 视图推导（非 live 为 null）
  const liveView = useMemo(
    () => (live ? liveToStageView(live, players, roomName) : null),
    [live, players, roomName],
  );

  // live 模式：对局加载中 / 出错（不再回落 mock 剧本，避免误导）
  const liveMode = serverBoard !== undefined;

  // 定时推进剧本（播完停帧）
  useEffect(() => {
    if (finished || !current) return undefined;
    const timer = setTimeout(() => setStep((s) => s + 1), current.duration);
    return () => clearTimeout(timer);
  }, [step, finished, current]);

  const speaker = current.speech
    ? players.find((p) => p.id === current.speech!.playerId)
    : undefined;

  // live 分支渲染：同舞台，数据源换成 server 事件流；未连上时展示等待/错误态
  if (liveMode) {
    if (!liveView) {
      return (
        <div className="flex h-dvh items-center justify-center bg-black">
          <VStack gap={3} className="items-center text-center">
            <p className="text-lg font-bold text-white">
              {liveError ?? '正在连接 server，等待引擎发牌……'}
            </p>
            {liveError ? (
              <p className="max-w-md text-sm text-white/60">
                请确认 server 已启动（cargo run -p server），且至少配置过一次选手模型。
              </p>
            ) : null}
            {onBack && (
              <button
                type="button"
                onClick={onBack}
                className="rounded-full bg-white/20 px-4 py-2 text-sm font-medium text-white hover:bg-white/30"
              >
                ← 返回大厅
              </button>
            )}
          </VStack>
        </div>
      );
    }
    const stage = liveView;
    return (
      <div className="relative h-dvh">
        <div className="absolute top-4 right-4 z-10 flex items-center gap-2">
          {onBack && (
            <button
              type="button"
              onClick={onBack}
              className="rounded-full bg-black/50 px-3 py-1.5 text-xs font-medium text-white shadow-card backdrop-blur-sm hover:bg-black/70"
            >
              ← 返回大厅
            </button>
          )}
          <span className="rounded-full bg-green/80 px-3 py-1.5 text-xs font-bold text-black shadow-card">
            LIVE · 真实对局
          </span>
        </div>
        <WerewolfStage
          title={roomName}
          players={stage.stagePlayers.map((p) =>
            toStagePlayer(p, {
              phase: stage.phase,
              phaseLabel: stage.phaseLabel,
              spotlightSeats: stage.spotlightSeats,
              speech:
                stage.speaker && stage.speechText
                  ? { playerId: stage.speaker.id, text: stage.speechText }
                  : undefined,
              deaths: [],
              duration: 0,
            }),
          )}
          phase={stage.phase}
          phaseLabel={stage.phaseLabel}
          caption={stage.caption}
          backgroundUrl="/werewolf/night-bg.jpg"
          fullBleed
        >
          <VStack gap={3} justify="center" className="h-full w-full">
            <HostPanel host={MOCK_HOST} message={stage.hostMessage} className="w-full max-w-xl" />
            {stage.speaker && stage.speechText ? (
              <SpeechPanel
                speaker={stage.speaker}
                text={stage.speechText}
                className="w-full max-w-xl"
              />
            ) : null}
            {liveError ? (
              <p className="rounded-full bg-red/80 px-4 py-1.5 text-xs font-medium text-white">
                {liveError}
              </p>
            ) : null}
          </VStack>
        </WerewolfStage>
      </div>
    );
  }

  return (
    <div className="relative h-dvh">
      {/* 演示控制：返回大厅 / 重播 */}
      <div className="absolute top-4 right-4 z-10 flex items-center gap-2">
        {onBack && (
          <button
            type="button"
            onClick={onBack}
            className="rounded-full bg-black/50 px-3 py-1.5 text-xs font-medium text-white shadow-card backdrop-blur-sm hover:bg-black/70"
          >
            ← 返回大厅
          </button>
        )}
        {finished && (
          <button
            type="button"
            onClick={() => setStep(0)}
            className="animate-fade-in rounded-full bg-yellow/90 px-3 py-1.5 text-xs font-bold text-black shadow-card hover:bg-yellow"
          >
            ↻ 重播流程
          </button>
        )}
      </div>

      <WerewolfStage
        title={roomName}
        players={players.map((p) => toStagePlayer(p, current))}
        phase={current.phase}
        phaseLabel={current.phaseLabel}
        caption={current.caption}
        backgroundUrl="/werewolf/night-bg.jpg"
        fullBleed
      >
        <VStack gap={3} justify="center" className="h-full w-full">
          {current.host ? (
            <HostPanel host={MOCK_HOST} message={current.host} className="w-full max-w-xl" />
          ) : null}
          {speaker && current.speech ? (
            <SpeechPanel speaker={speaker} text={current.speech.text} className="w-full max-w-xl" />
          ) : null}
        </VStack>
      </WerewolfStage>
    </div>
  );
}

export type { FlowStep };
