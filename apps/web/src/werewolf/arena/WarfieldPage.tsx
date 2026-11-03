/**
 * WarfieldPage —— 对局实况页（观战）：SSE 事件流驱动，页面零轮询。
 *
 * 数据链路：RoomSetup 预热开局（后端 LLM 推理先行）→ 本页 useGameStream 订阅
 * /api/game/stream → WerewolfGame 基类去重/回放 → projectStage 投影到 WerewolfStage。
 * 介绍期未进本页的事件不丢：SSE 每帧都是全量快照，进入后按 seq 追平。
 */
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { projectStage } from '@agent-packages/game';
import { HStack, VStack, WerewolfStage, type WerewolfPlayer } from '@agent-packages/shared-ui';
import { CaptionBar } from '../CaptionBar';
import { MvpPanel } from './MvpPanel';
import { SpeechPanel } from './SpeechPanel';
import { speak, stopSpeaking } from '../speech';
import { buildPlayers, dealtFromServer, dealGame, type BoardDef } from '../boards';
import { ROLE_LABEL } from '../lobby/roles';
import type { GameStatusDto } from '../../apis/types';
import type { Player } from '../types';
import { useGameStream } from './useGameStream';
import {
  CountdownOverlay,
  NIGHTFALL_MS,
  NightfallOverlay,
  OpeningCeremony,
  type CeremonyPlayer,
} from '@agent-packages/animation';
import { failureCopy, WaitingNotice, waitingTitle } from './failures';
import { RecordButton, useRecorder } from './useRecorder';

/** 领域 Player → 舞台座位：夜晚非睁眼者深度压暗，睁眼/发言全彩，死亡态置灰 */
function toStagePlayer(
  p: Player,
  stage: NonNullable<ReturnType<typeof projectStage<Player>>>,
): WerewolfPlayer {
  const spotlighted = stage.spotlightSeats.includes(p.id);
  const speaking = stage.speaker?.id === p.id;
  return {
    seat: p.id,
    name: p.name,
    avatarSrc: p.avatarSrc,
    roleLabel: p.role === 'werewolf' ? '狼人' : ROLE_GOOD_LABEL[p.role],
    roleTone: p.role === 'werewolf' ? 'wolf' : 'good',
    alive: p.status === 'alive',
    speaking,
    spotlight: spotlighted && !speaking,
    dimmed: stage.phase === 'night' && !spotlighted && !speaking && p.status === 'alive',
  };
}

const ROLE_GOOD_LABEL: Record<Exclude<Player['role'], 'werewolf'>, string> = {
  seer: '预言家',
  witch: '女巫',
  guard: '守卫',
  hunter: '猎人',
  villager: '平民',
};

/** 角色 → 身份牌文案/色调（仪式亮牌用，与舞台 RoleBadge 同源） */
const CEREMONY_ROLE: Record<Player['role'], { label: string; tone: CeremonyPlayer['roleTone'] }> = {
  werewolf: { label: ROLE_LABEL.werewolf, tone: 'wolf' },
  seer: { label: ROLE_GOOD_LABEL.seer, tone: 'good' },
  witch: { label: ROLE_GOOD_LABEL.witch, tone: 'good' },
  guard: { label: ROLE_GOOD_LABEL.guard, tone: 'good' },
  hunter: { label: ROLE_GOOD_LABEL.hunter, tone: 'good' },
  villager: { label: ROLE_GOOD_LABEL.villager, tone: 'good' },
};

function toCeremonyPlayer(p: Player): CeremonyPlayer {
  const role = CEREMONY_ROLE[p.role];
  return {
    seat: p.id,
    name: p.name,
    avatarSrc: p.avatarSrc,
    roleLabel: role.label,
    roleTone: role.tone,
  };
}

function toCeremonyRoster(p: Player) {
  return { seat: p.id, name: p.name, avatarSrc: p.avatarSrc };
}

/**
 * 对局实况页：订阅 server 事件流（SSE），实时投影到舞台。
 * 新开局自动 5-4-3-2-1 倒计时；观战模式点「开始回放」从头逐条回放，可一键追平直播。
 */
export function WarfieldPage({
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
  /** 房间号：SSE 请求全部带 ?room= 定位到该房间的对局（多槽位防串局） */
  roomCode?: string;
  onBack?: () => void;
}) {
  const players = useMemo(() => playersOverride ?? buildPlayers(board), [board, playersOverride]);
  const locator = useMemo(
    () => (roomCode !== undefined ? { room: roomCode } : undefined),
    [roomCode],
  );
  const { game, snapshot, phase, failure: streamFailure } = useGameStream(locator);
  const { events, playhead, countdown, replayStarted } = snapshot;
  const live = snapshot.status;
  const liveError = snapshot.error;
  /** 选手名/主持人 → Edge 音色（每局随机分配；主持人固定专业新闻腔）；无分配（历史局）不启用 */
  const voiceMap = useMemo<Record<string, string> | undefined>(() => {
    if (!live?.seat_voices?.length) return undefined;
    const map: Record<string, string> = { 主持人: 'zh-CN-YunyangNeural' };
    live.seats.forEach((name, i) => {
      if (name && live.seat_voices?.[i]) map[name] = live.seat_voices[i];
    });
    return map;
  }, [live]);
  const failure = streamFailure;
  const failureView = failure !== null ? failureCopy(failure) : null;

  // live 身份牌以 server 真实发牌为准：本地 dealGame 只是预热回退，
  // 不同步的话狼聊日志里的"3号（狼）"会和座位牌（平民）对不上
  const livePlayers = useMemo(() => {
    if (live?.seats === undefined || live.seats.length === 0) return players;
    return dealtFromServer(board, live.seats, live.seat_roles ?? [], roomCode).players;
  }, [board, live, players, roomCode]);

  // 新开局：先演发牌仪式（洗牌→发牌→亮牌→转暗），仪式收场才请求基类开演
  //（倒计时在暗屏上出数字，结束后 NightfallOverlay 由暗到亮）；观战回放立即点火
  const [ceremonyDone, setCeremonyDone] = useState(false);
  useEffect(() => {
    if (spectate || ceremonyDone) game.begin();
  }, [game, spectate, ceremonyDone]);

  // 仪式发牌数据：server seat_roles 事实源（SSE 全量快照自带）；server 不可达回退本地随机，
  // 未就绪（预热中）传 null——仪式只演入座静场，不发牌
  const ceremonyPlayers = useMemo<CeremonyPlayer[] | null>(() => {
    if (spectate || ceremonyDone) return null;
    const seatsReady =
      live?.seats?.length === board.total && (live.seat_roles?.length ?? 0) === board.total;
    if (seatsReady && live) {
      return dealtFromServer(board, live.seats, live.seat_roles ?? [], roomCode).players.map(
        toCeremonyPlayer,
      );
    }
    if (failure !== null) {
      return dealGame(
        board,
        players.map(({ name, avatarSrc }) => ({ name, avatarSrc: avatarSrc ?? '' })),
      ).players.map(toCeremonyPlayer);
    }
    return null;
  }, [spectate, ceremonyDone, live, board, roomCode, failure, players]);

  // GSAP 时间线结束回调；useCallback 保引用稳定，避免 useGSAP 依赖变化重启动画
  const finishCeremony = useCallback(() => setCeremonyDone(true), []);

  // 回放进度全部来自基类快照：events 已按 seq 去重增量累积，playhead 由基类节拍器逐条推进
  // （倒计时 / 缓冲等待 / 回放点火也都在基类内，web 只消费快照与调用控制方法）
  const totalEvents = events.length;
  const caughtUp = countdown === null && playhead >= totalEvents - 1;

  // live 舞台视图 = 已播事件前缀的投影
  const stage = useMemo(() => {
    if (!live || playhead < 0) return null;
    // 第 4 参：未播的下一条事件，供主持词衔接（如狼刀后接「狼人请闭眼」）
    return projectStage(events.slice(0, playhead + 1), livePlayers, roomName, events[playhead + 1]);
  }, [live, events, playhead, livePlayers, roomName]);

  // 连接空窗静默：首帧是全量快照，通常几百毫秒内就位；等待文案延迟 600ms 才出现，
  // 避免冷启动 / 倒计时结束到首个事件投影之间闪一下「正在连接」。已收到事件
  // （dealPending=false，发牌完成）时不显示「正在连接」——那是节拍器还没走到首条事件。
  // 状态只置真不清零：显隐由渲染期用 dealPending 门控，避免 effect 里同步 setState
  const [connectNoticeShown, setConnectNoticeShown] = useState(false);
  const dealPending = stage === null && countdown === null && totalEvents === 0;
  useEffect(() => {
    if (failure !== null || phase === 'reconnecting' || !dealPending) return undefined;
    const timer = window.setTimeout(() => setConnectNoticeShown(true), 600);
    return () => window.clearTimeout(timer);
  }, [failure, phase, dealPending]);

  // 天黑过场：倒计时归零（→ null）触发狼嚎 + 黑幕渐隐揭示舞台；观战/回放无倒计时不触发。
  // 狼嚎来源：美国 Fish & Wildlife Service 录音（美国政府作品，公共领域），已截段淡入淡出
  const [nightfall, setNightfall] = useState(false);
  const prevCountdown = useRef(snapshot.countdown);
  useEffect(() => {
    const was = prevCountdown.current;
    prevCountdown.current = snapshot.countdown;
    if (was === null || snapshot.countdown !== null) return undefined;
    setNightfall(true);
    const howl = new Audio('/werewolf/wolf-howl.mp3');
    howl.volume = 0.6;
    void howl.play().catch(() => undefined); // 自动播放策略拦截时静默降级
    const timer = window.setTimeout(() => setNightfall(false), NIGHTFALL_MS); // 兜底：GSAP 时间线结束会提前回调 finishNightfall
    return () => {
      window.clearTimeout(timer);
      howl.pause(); // 提前离页就收声
    };
  }, [snapshot.countdown]);

  // GSAP 时间线结束回调；useCallback 保引用稳定，避免 useGSAP 依赖变化重启动画
  const finishNightfall = useCallback(() => setNightfall(false), []);

  // 结算：server finish_game 后 winner 出现 → 展示结算浮层 + 主持人宣判语音
  const winner = live?.winner ?? null;
  useEffect(() => {
    // 回放未追平（还没点结束/演完）不抢跑宣判语音，避免封面页就开始播报胜负
    if (!winner || !caughtUp) return undefined;
    const line =
      winner === 'good'
        ? '天亮了！好人阵营找出所有狼人，胜利！'
        : '夜幕降临，狼人阵营潜伏至最后，胜利！';
    const hostVoice = voiceMap?.主持人;
    void speak(hostVoice ?? '主持人', line, hostVoice ? 'edge' : undefined).catch(() => undefined);
    return () => stopSpeaking();
  }, [winner, caughtUp, voiceMap]);

  // 对局录制：介绍页已自动开启（推流单例跨页面接管），终局（胜负分出）自动停录
  const recorder = useRecorder();
  useEffect(() => {
    if (winner && recorder.recording) recorder.stop();
  }, [winner, recorder]);

  // 开局仪式：规则讲完直进对局页，舞台先入座（无身份），真实发牌就绪后开演
  if (!spectate && !ceremonyDone) {
    return (
      <div className="relative h-dvh overflow-hidden bg-arena">
        <OpeningCeremony
          roster={players.map(toCeremonyRoster)}
          players={ceremonyPlayers}
          onComplete={finishCeremony}
          backgroundUrl="/werewolf/night-bg.jpg"
        />
        {/* 仪式期间 server 故障：静场底部就地提示（发牌回退本地随机已兜底演出） */}
        {failureView && (
          <div className="absolute inset-x-0 bottom-6 z-40 flex justify-center">
            <WaitingNotice show title={failureView.title} hint={failureView.hint} onBack={onBack} />
          </div>
        )}
      </div>
    );
  }

  // 等待态：未连上事件流（倒计时 / 观战回放封面 / 错误）
  if (!stage) {
    return (
      <div className="relative flex h-dvh items-center justify-center overflow-hidden bg-arena">
        {countdown !== null ? (
          <CountdownOverlay value={countdown} waiting={totalEvents === 0} />
        ) : nightfall ? (
          <NightfallOverlay onDone={finishNightfall} />
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
          <WaitingNotice
            show={
              failureView !== null ||
              phase === 'reconnecting' ||
              (connectNoticeShown && dealPending)
            }
            title={waitingTitle(failureView?.title ?? null, phase === 'reconnecting')}
            hint={failureView?.hint}
            onBack={onBack}
          />
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
          <RecordButton
            recording={recorder.recording}
            seconds={recorder.seconds}
            error={recorder.error}
            onStart={() => void recorder.start(roomName)}
            onStop={recorder.stop}
          />
        </div>
        <WerewolfStage
          title={roomName}
          players={stage.stagePlayers.map((p) => toStagePlayer(p, stage))}
          phase={stage.phase}
          phaseLabel={stage.phaseLabel}
          caption={stage.caption}
          backgroundUrl="/werewolf/night-bg.jpg"
          fullBleed
        >
          <VStack gap={4} justify="between" className="h-full w-full py-2">
            {stage.wolfChat.length > 0 ? (
              // 狼人内部交流：同晚商议段并排同屏展示，不播语音（CaptionBar 只走主持人词）
              <HStack gap={3} className="w-full max-w-2xl items-stretch justify-center">
                {stage.wolfChat.map(({ player, text }) => (
                  <SpeechPanel
                    key={player.id}
                    speaker={player}
                    text={text}
                    className="min-w-0 flex-1"
                  />
                ))}
              </HStack>
            ) : stage.speaker && stage.speechText ? (
              <SpeechPanel
                speaker={stage.speaker}
                text={stage.speechText}
                className="w-full max-w-xl"
              />
            ) : null}
            <CaptionBar
              voice={stage.speaker?.name ?? '主持人'}
              text={stage.speechText ?? stage.hostMessage}
              voices={voiceMap}
              // 狼聊：依次用各狼人自己的音色播商议内容
              segments={
                stage.wolfChat.length > 0
                  ? stage.wolfChat.map(({ player, text: t }) => ({ voice: player.name, text: t }))
                  : undefined
              }
              // 说完才演下一条：播报完成（含降级）驱动基类节拍器提前推进
              onAllSpoken={() => game.speechDone()}
            />
            {liveError ? (
              <p className="rounded-full bg-red/80 px-4 py-1.5 text-xs font-medium text-arena-text">
                {liveError}
              </p>
            ) : null}
          </VStack>
        </WerewolfStage>
      </div>
      {/* 天黑过场：叠在舞台上方，黑幕渐隐揭示选手（倒计时结束的短暂黑屏期） */}
      {nightfall ? <NightfallOverlay onDone={finishNightfall} /> : null}
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
            {/* 赛后 MVP：SSE 帧全量快照自动刷新到 done；null（手动终局/中断局）不渲染 */}
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

// 供类型引用：GameStatusDto 在快照泛型里使用（保持导入被消费）
export type { GameStatusDto };
