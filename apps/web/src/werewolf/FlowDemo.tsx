import { useEffect, useMemo, useRef, useState, useSyncExternalStore } from 'react';
import {
  MIN_START_BUFFER_EVENTS,
  POLL_INTERVAL_MS,
  WerewolfGame,
  projectStage,
} from '@agent-packages/game';
import { VStack, WerewolfStage, cn, type WerewolfPlayer } from '@agent-packages/shared-ui';
import { CaptionBar } from './CaptionBar';
import { HostPanel } from './HostPanel';
import { MOCK_HOST } from './mock';
import { MvpPanel } from './MvpPanel';
import { SpeechPanel } from './SpeechPanel';
import { speak, stopSpeaking } from './speech';
import { buildPlayers, dealtFromServer, type BoardDef } from './boards';
import {
  fetchGameStatus,
  startGame,
  toServerBoard,
  type GameLocator,
  type GameStatusDto,
} from './liveClient';
import type { Player } from './types';
import { RecordButton, useRecorder } from './useRecorder';

/** 夜间动作标记：刀 / 盾 / 救 / 查验结果（demo 展示件，色语义与 RoleBadge 对齐） */
function SeatMark({ kind }: { kind: 'kill' | 'guard' | 'good' | 'wolf' | 'save' }) {
  const meta = {
    kill: { text: '刀', cls: 'bg-red text-arena-text' },
    guard: { text: '盾', cls: 'bg-blue text-arena-text' },
    good: { text: '好', cls: 'bg-green text-arena' },
    wolf: { text: '狼', cls: 'bg-red text-arena-text' },
    save: { text: '救', cls: 'bg-green text-arena' },
  }[kind];
  return (
    <span
      className={cn(
        'flex size-7 items-center justify-center rounded-full text-xs font-black shadow-card',
        'ring-1 ring-arena-line',
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

/** 开局倒计时浮层：5-4-3-2-1 逐秒过场（stage 就绪前在等待页展示，倒数完才开演） */
function CountdownOverlay({ value, waiting }: { value: number; waiting: boolean }) {
  return (
    <div className="absolute inset-0 z-30 flex flex-col items-center justify-center gap-6 bg-arena/70 backdrop-blur-sm">
      <p className="animate-fade-in text-lg font-medium text-arena-muted">
        对局即将开始，请各位选手就位
      </p>
      <p
        key={value}
        className="animate-zoom-in text-8xl font-black tracking-wider text-arena-text drop-shadow-[0_4px_24px_rgb(0_0_0/0.6)]"
      >
        {value > 0 ? value : '开始！'}
      </p>
      <p className="animate-fade-in text-sm text-arena-muted">
        {waiting ? '选手正在入场，主持人整理牌局中……' : '主持人即将入场 · 天黑请闭眼……'}
      </p>
    </div>
  );
}

/**
 * 房间对局演示：按板子生成玩家与剧本（或接收发牌结果），定时推进“天黑→狼刀→查验→[守卫]→[女巫]→天亮”，
 * 纯前端编排（引擎接入后由事件流驱动同一舞台）。播完停在末帧，可重播。
 */
export function FlowDemo({
  board,
  roomName,
  players: playersOverride,
  spectate = false,
  roomCode,
  onBack,
}: {
  board: BoardDef;
  roomName: string;
  /** 发牌结果注入（RoomSetup 随机分配后传入）；缺省取选手池前 N 人 */
  players?: Player[];
  /** 大厅观战进行中对局：跳过倒计时仪式，直接追平实时进度 */
  spectate?: boolean;
  /** 房间号：live 请求全部带 ?room= 定位到该房间的对局（多槽位防串局） */
  roomCode?: string;
  onBack?: () => void;
}) {
  const players = useMemo(() => playersOverride ?? buildPlayers(board), [board, playersOverride]);
  // 对局基类：事件队列 / 回放节奏 / 倒计时全部由 @agent-packages/game 承担，web 只负责拉数据喂入
  const game = useMemo(() => new WerewolfGame<GameStatusDto>(), []);
  useEffect(() => () => game.destroy(), [game]);
  const snapshot = useSyncExternalStore(game.subscribe, game.getSnapshot);
  const { events, playhead, countdown, replayStarted } = snapshot;
  const live = snapshot.status;
  const liveError = snapshot.error;
  // live 身份牌以 server 真实发牌为准：本地 dealGame 只是预热回退，
  // 不同步的话狼聊日志里的“3号（狼）”会和座位牌（平民）对不上
  const livePlayers = useMemo(() => {
    if (live?.seats === undefined || live.seats.length === 0) return players;
    return dealtFromServer(board, live.seats, live.seat_roles ?? [], roomCode).players;
  }, [board, live, players, roomCode]);
  const startedRef = useRef(false);
  const serverBoard = toServerBoard(board.id);

  // live 模式：开局（幂等，已在对局则直接观战）+ 轮询事件流；全部带 ?room= 定位。
  // 拉到的数据 ingest 喂入基类（内部按 seq 去重累积），失败 fail 透出——对接后端只到这一层
  useEffect(() => {
    const target = toServerBoard(board.id);
    if (!target || startedRef.current) return undefined;
    startedRef.current = true;
    const seatNames = players.map((p) => p.name);
    const locator: GameLocator | undefined = roomCode ? { room: roomCode } : undefined;
    let startTried = false; // 只在首次失败时尝试开局，避免瞬时不可达反复重建对局
    const poll = async () => {
      try {
        const next = await fetchGameStatus(locator);
        game.ingest({ events: next.events, status: next });
      } catch {
        // 定位不到对局（多槽位下无局 → 404）：尝试开局（创建路径；观战路径会拿到房间不存在错误）
        if (startTried) return;
        startTried = true;
        try {
          const started = await startGame(target, seatNames, roomCode);
          game.ingest({ events: started.events, status: started });
        } catch (startErr) {
          game.fail(startErr instanceof Error ? startErr.message : '连接 server 失败');
        }
      }
    };
    void poll();
    const timer = setInterval(poll, POLL_INTERVAL_MS);
    // 后台 tab 被 Chromium 深度节流（interval 压到 1/分钟）：回到前台立即拉取补齐事件
    const onVisible = () => {
      if (document.visibilityState === 'visible') void poll();
    };
    document.addEventListener('visibilitychange', onVisible);
    return () => {
      clearInterval(timer);
      document.removeEventListener('visibilitychange', onVisible);
      startedRef.current = false;
    };
  }, [board, players, roomCode, game]);

  // 新开局：请求基类开演（事件就绪后自动 5-4-3-2-1 倒计时）；观战回放由「开始回放」按钮点火
  useEffect(() => {
    if (serverBoard !== undefined && !spectate) game.begin();
  }, [game, serverBoard, spectate]);
  const script = useMemo(() => buildScript(board, roomName, players), [board, roomName, players]);
  const [step, setStep] = useState(0);
  const current = script[step];
  const finished = step === script.length - 1;

  // 回放进度全部来自基类快照：events 已按 seq 去重增量累积，playhead 由基类节拍器逐条推进
  // （倒计时 / 缓冲等待 / 回放点火也都在基类内，web 只消费快照与调用控制方法）
  const totalEvents = events.length;
  const caughtUp = countdown === null && playhead >= totalEvents - 1;
  /** 快进：直接追平服务端最新进度 */
  const skipToLive = () => game.skipToLive();

  // live 舞台视图 = 已播事件前缀的投影（非 live 为 null）
  const stage = useMemo(() => {
    if (!live || playhead < 0) return null;
    return projectStage(events.slice(0, playhead + 1), livePlayers, roomName);
  }, [live, events, playhead, livePlayers, roomName]);

  // live 模式：对局加载中 / 出错（不再回落 mock 剧本，避免误导）
  const liveMode = serverBoard !== undefined;

  // 结算：server finish_game 后 winner 出现 → 展示结算浮层 + 主持人宣判语音
  const winner = liveMode ? (live?.winner ?? null) : null;
  useEffect(() => {
    // 回放未追平（还没点结束/演完）不抢跑宣判语音，避免封面页就开始播报胜负
    if (!winner || !caughtUp) return undefined;
    const line =
      winner === 'good'
        ? '天亮了！好人阵营找出所有狼人，胜利！'
        : '夜幕降临，狼人阵营潜伏至最后，胜利！';
    void speak('主持人', line).catch(() => undefined);
    return () => stopSpeaking();
  }, [winner, caughtUp]);

  // 对局录制：手动启停，终局（胜负分出）自动停录并下载 webm
  const recorder = useRecorder(roomName);
  useEffect(() => {
    if (winner && recorder.recording) recorder.stop();
  }, [winner, recorder]);

  // 定时推进剧本（播完停帧）：step 驱动 current 变化，无需入 deps
  useEffect(() => {
    if (finished || !current) return undefined;
    const timer = setTimeout(() => setStep((s) => s + 1), current.duration);
    return () => clearTimeout(timer);
  }, [finished, current]);

  const speaker = current.speech
    ? players.find((p) => p.id === current.speech!.playerId)
    : undefined;

  // live 分支渲染：同舞台，数据源换成 server 事件流；未连上时展示等待/错误态
  if (liveMode) {
    if (!stage) {
      return (
        <div className="relative flex h-dvh items-center justify-center overflow-hidden bg-arena">
          {countdown !== null ? (
            <CountdownOverlay value={countdown} waiting={totalEvents < MIN_START_BUFFER_EVENTS} />
          ) : spectate && totalEvents > 0 && !replayStarted ? (
            <VStack
              gap={4}
              className="animate-fade-in items-center rounded-3xl border border-arena-line bg-arena/60 px-12 py-10 text-center shadow-card"
            >
              <span className="text-5xl" aria-hidden>
                🎬
              </span>
              <p className="text-lg font-bold text-arena-text">{roomName} · 对局回放</p>
              <p className="text-sm text-arena-muted">
                已录制 {totalEvents} 条事件 · 点击开始后从头逐条回放
              </p>
              <button
                type="button"
                onClick={() => game.startReplay()}
                className="rounded-full bg-arena-gold/90 px-6 py-2 text-sm font-bold text-arena shadow-card transition-colors duration-200 hover:bg-arena-gold"
              >
                ▶ 开始回放
              </button>
              {onBack && (
                <button
                  type="button"
                  onClick={onBack}
                  className="text-xs text-arena-muted hover:text-arena-text"
                >
                  ← 返回大厅
                </button>
              )}
            </VStack>
          ) : (
            <VStack gap={3} className="items-center text-center">
              <p className="text-lg font-bold text-arena-text">
                {liveError ?? '正在连接 server，等待引擎发牌……'}
              </p>
              {liveError ? (
                <p className="max-w-md text-sm text-arena-muted">
                  请确认 server 已启动（cargo run -p server），且至少配置过一次选手模型。
                </p>
              ) : null}
              {onBack && (
                <button
                  type="button"
                  onClick={onBack}
                  className="rounded-full bg-arena-raised px-4 py-2 text-sm font-medium text-arena-text hover:bg-arena-raised"
                >
                  ← 返回大厅
                </button>
              )}
            </VStack>
          )}
        </div>
      );
    }
    return (
      <div className="relative flex h-dvh overflow-hidden">
        <div className="relative min-w-0 flex-1">
          <div className="absolute top-4 right-4 z-10 flex items-center gap-2">
            {onBack && (
              <button
                type="button"
                onClick={onBack}
                className="rounded-full bg-arena/50 px-3 py-1.5 text-xs font-medium text-arena-text shadow-card backdrop-blur-sm hover:bg-arena/70"
              >
                ← 返回大厅
              </button>
            )}
            <span className="rounded-full bg-green/80 px-3 py-1.5 text-xs font-bold text-arena shadow-card">
              LIVE · 真实对局
            </span>
            <RecordButton
              recording={recorder.recording}
              seconds={recorder.seconds}
              error={recorder.error}
              onStart={() => void recorder.start()}
              onStop={recorder.stop}
            />
            {/* 服务端超前时展示回放进度，可一键追平 */}
            {!caughtUp ? (
              <button
                type="button"
                onClick={skipToLive}
                className="rounded-full bg-arena-raised px-3 py-1.5 text-xs font-medium text-arena-text shadow-card backdrop-blur-sm transition-colors duration-200 hover:bg-arena-raised"
              >
                {spectate && replayStarted
                  ? `⏭ 结束回放 · 跳到最新（${playhead + 2}/${totalEvents}）`
                  : `⏩ 回放中 ${playhead + 2}/${totalEvents} · 跳到最新`}
              </button>
            ) : (
              // 追平直播后等待服务端下一步（LLM 决策需数秒）：给出明确状态，避免看起来像卡死
              !winner && (
                <span className="animate-fade-in rounded-full bg-arena-raised px-3 py-1.5 text-xs font-medium text-arena-muted backdrop-blur-sm">
                  🔴 跟随直播 · 等待选手行动…
                </span>
              )
            )}
          </div>
          {/* 对局日志已改为 flex 右栏，不再叠加在座位上 */}
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
              <CaptionBar
                voice={stage.speaker?.name ?? '主持人'}
                text={stage.speechText ?? stage.hostMessage}
                preamble={
                  stage.speechText && stage.speaker
                    ? `有请 ${stage.speaker.id} 号玩家，${stage.speaker.name}，请发言。`
                    : undefined
                }
              />
              {liveError ? (
                <p className="rounded-full bg-red/80 px-4 py-1.5 text-xs font-medium text-arena-text">
                  {liveError}
                </p>
              ) : null}
            </VStack>
          </WerewolfStage>
        </div>
        {/* 结算浮层：胜负 + 全场身份回顾（server finish_game 已落库）；回放未追平时不抢跑 */}
        {winner && caughtUp ? (
          <div className="animate-fade-in absolute inset-0 z-20 flex items-center justify-center bg-arena/80 backdrop-blur-md">
            <VStack
              gap={5}
              className="items-center rounded-3xl border border-arena-line bg-arena/60 px-12 py-10 text-center shadow-card"
            >
              <span className="text-6xl" aria-hidden>
                {winner === 'good' ? '🌞' : '🐺'}
              </span>
              <h2 className="text-3xl font-black tracking-wide text-arena-text">
                {winner === 'good' ? '好人阵营 · 胜利' : '狼人阵营 · 胜利'}
              </h2>
              <p className="text-sm text-arena-muted">鏖战 {live?.day ?? '?'} 天 · 全场身份回顾</p>
              {live?.stale ? (
                <span className="rounded-full bg-arena-raised px-3 py-0.5 text-xs font-bold text-arena-muted">
                  ⚠ 对局已中断（服务器重启）
                </span>
              ) : null}
              <ul className="grid max-w-lg grid-cols-2 gap-x-8 gap-y-1.5 text-left text-sm">
                {(live?.seats ?? []).map((name, i) => (
                  <li key={name} className="flex items-center justify-between gap-3">
                    <span className="text-arena-muted">
                      {i + 1}. {name}
                    </span>
                    <span className="font-mono text-xs text-arena-muted">
                      {live?.seat_roles?.[i] ?? '?'}
                    </span>
                  </li>
                ))}
              </ul>
              {/* 赛后 MVP：pending 时随既有 status 轮询自动刷新到 done；null（手动终局/中断局）不渲染 */}
              <MvpPanel mvp={live?.mvp} />
              {onBack && (
                <button
                  type="button"
                  onClick={onBack}
                  className="mt-2 rounded-full bg-arena-gold/90 px-5 py-2 text-sm font-bold text-arena shadow-card hover:bg-arena-gold"
                >
                  回到大厅 →
                </button>
              )}
            </VStack>
          </div>
        ) : null}
      </div>
    );
  }

  return (
    <div className="relative h-dvh overflow-hidden">
      {/* 演示控制：返回大厅 / 重播 / 录制 */}
      <div className="absolute top-4 right-4 z-10 flex items-center gap-2">
        <RecordButton
          recording={recorder.recording}
          seconds={recorder.seconds}
          error={recorder.error}
          onStart={() => void recorder.start()}
          onStop={recorder.stop}
        />
        {onBack && (
          <button
            type="button"
            onClick={onBack}
            className="rounded-full bg-arena/50 px-3 py-1.5 text-xs font-medium text-arena-text shadow-card backdrop-blur-sm hover:bg-arena/70"
          >
            ← 返回大厅
          </button>
        )}
        {finished && (
          <button
            type="button"
            onClick={() => setStep(0)}
            className="animate-fade-in rounded-full bg-arena-gold/90 px-3 py-1.5 text-xs font-bold text-arena shadow-card hover:bg-arena-gold"
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
          <CaptionBar
            voice={current.speech ? (speaker?.name ?? '主持人') : '主持人'}
            text={current.speech?.text ?? current.host ?? ''}
            preamble={
              current.speech
                ? `有请 ${current.speech.playerId} 号玩家，${speaker?.name ?? ''}，请发言。`
                : undefined
            }
          />
        </VStack>
      </WerewolfStage>
    </div>
  );
}

export type { FlowStep };
