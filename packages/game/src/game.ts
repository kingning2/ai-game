/**
 * WerewolfGame：对局前端基类（框架无关、传输无关、无 React 依赖）。
 *
 * 职责边界（见 rules/common/package-rules.md）：
 * - 本类不接后端：不发请求、不轮询、不知道端点与 DTO。调用方（web 等）自己拉数据，
 *   拉到后 ingest() 喂进来，失败 fail() 透出；
 * - 本类只管「演到哪一条」：事件按 seq 去重增量累积为回放队列，playhead 逐条推进
 *   （每类事件按 playbackMs 停留）；新开局 begin() 先 5-4-3-2-1 倒计时再开演；
 *   观战/回放 startReplay() 从头逐条演出，skipToLive() 一键追平；
 * - 快照订阅：subscribe(listener) 拿不可变快照，配 useSyncExternalStore 等渲染层使用。
 *
 * 舞台投影（projectStage）与主持词（hostSay）是纯函数，见 playback.ts。
 */
import type { JsonValue } from '@agent-packages/shared';
import { playbackMs } from './playback';
import type { GameEventDto, GameFeed } from './types';

/** 开局倒计时秒数：事件就绪后先倒数再开演，让主持仪式感先入戏 */
export const COUNTDOWN_FROM_SECONDS = 5;

/** 状态轮询间隔（ms）——调用方若用轮询拉数据，建议用这个节奏 */
export const POLL_INTERVAL_MS = 1500;

/** 不可变快照：每次状态变化生成新对象，渲染层整体替换 */
export type WerewolfGameSnapshot<TStatus = JsonValue> = {
  /** 数据源最新状态原样透传（null = 尚未喂过数据） */
  status: TStatus | null;
  /** 调用方透出的错误（fail() 的 message） */
  error: string | null;
  /** 按 seq 去重增量累积的回放队列 */
  events: GameEventDto[];
  /** 已演到的事件下标（-1 = 尚未开演） */
  playhead: number;
  /** 开局倒计时当前秒数（null = 不在倒计时） */
  countdown: number | null;
  /** 观战回放是否已点「开始回放」 */
  replayStarted: boolean;
};

export class WerewolfGame<TStatus = JsonValue> {
  private snapshot: WerewolfGameSnapshot<TStatus> = {
    status: null,
    error: null,
    events: [],
    playhead: -1,
    countdown: null,
    replayStarted: false,
  };
  private readonly listeners = new Set<(snapshot: WerewolfGameSnapshot<TStatus>) => void>();

  private maxSeq = 0;
  /** begin() 已请求：等首批事件到达后自动起倒计时 */
  private begun = false;
  /** 倒计时只起一次（防重入） */
  private countdownStarted = false;
  /** 演出进行中（倒计时结束 / startReplay / skipToLive 置位） */
  private playing = false;
  /** destroy() 后丢弃后续 ingest 的节拍排程（在途轮询回调仍可能调进来） */
  private destroyed = false;
  /** 实例是否已销毁（渲染层 StrictMode 重挂时据此换新实例） */
  get isDestroyed(): boolean {
    return this.destroyed;
  }
  private beatTimer: ReturnType<typeof setTimeout> | undefined;
  private countdownTimer: ReturnType<typeof setTimeout> | undefined;

  // —— 订阅（箭头属性：实例级稳定引用，适配 useSyncExternalStore） ——

  readonly subscribe = (
    listener: (snapshot: WerewolfGameSnapshot<TStatus>) => void,
  ): (() => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };

  readonly getSnapshot = (): WerewolfGameSnapshot<TStatus> => this.snapshot;

  // —— 数据输入（调用方拉到数据后喂入） ——

  /**
   * 喂入数据源的全量事件流：内部按 seq 只取新鲜部分增量累积（可反复调，服务端可任意超前）。
   * status 原样透传；begin() 挂起中且首批事件就绪时自动起倒计时。
   */
  readonly ingest = (feed: GameFeed<TStatus>): void => {
    if (this.destroyed) return;
    const fresh = feed.events.filter((e) => e.seq > this.maxSeq);
    if (fresh.length > 0) this.maxSeq = fresh[fresh.length - 1].seq;
    this.#patch({
      status: feed.status ?? this.snapshot.status,
      error: null,
      events: fresh.length > 0 ? [...this.snapshot.events, ...fresh] : this.snapshot.events,
    });
    this.#maybeStartCountdown();
    this.#scheduleBeat();
  };

  /** 数据拉取失败透出（调用方在 catch 里调，message 直达 UI） */
  readonly fail = (message: string): void => {
    this.#patch({ error: message });
  };

  // —— 回放控制 ——

  /** 新开局路径：请求开演（幂等）。事件未就绪时挂起，首批 ingest 到达后自动 5-4-3-2-1 倒计时 */
  readonly begin = (): void => {
    if (this.begun || this.countdownStarted || this.snapshot.replayStarted) return;
    this.begun = true;
    this.#maybeStartCountdown();
  };

  /** 观战/回放路径：点「开始回放」后从头逐条演出（不追平，让演出按自己的节奏走） */
  readonly startReplay = (): void => {
    if (this.snapshot.replayStarted || this.snapshot.events.length === 0) return;
    this.begun = false; // 抑制倒计时路径
    this.countdownStarted = true;
    this.#teardownCountdown();
    this.playing = true;
    this.#patch({ replayStarted: true, playhead: 0, countdown: null });
    this.#scheduleBeat();
  };

  /** 快进：直接追平数据源最新进度（「结束回放 / 跳到最新」） */
  readonly skipToLive = (): void => {
    const last = this.snapshot.events.length - 1;
    if (last < 0 || this.snapshot.playhead >= last) return;
    this.playing = true;
    this.#teardownBeat();
    this.#patch({ playhead: last });
  };

  /** 停止一切定时器与订阅（组件卸载 / 换局前调用） */
  destroy(): void {
    this.destroyed = true;
    this.#teardownBeat();
    this.#teardownCountdown();
    this.listeners.clear();
  }

  // —— 内部：倒计时仪式（新开局路径） ——

  #maybeStartCountdown(): void {
    const { events, playhead, countdown, replayStarted } = this.snapshot;
    if (!this.begun || this.countdownStarted || replayStarted) return;
    if (events.length === 0 || playhead >= 0 || countdown !== null) return;
    this.countdownStarted = true;
    this.#patch({ countdown: COUNTDOWN_FROM_SECONDS });
    this.#tickCountdown();
  }

  /** 倒计时节奏：5→1 每秒减一；归零即开演——已有事件立刻按序播出，
   *  尚无事件则等首批 ingest 到达再起节拍（ingest 每次都会重排 #scheduleBeat） */
  #tickCountdown(): void {
    const current = this.snapshot.countdown;
    if (current === null) return;
    if (current > 0) {
      this.countdownTimer = setTimeout(() => {
        this.countdownTimer = undefined;
        if (this.snapshot.countdown === null) return;
        this.#patch({ countdown: this.snapshot.countdown - 1 });
        this.#tickCountdown();
      }, 1000);
      return;
    }
    this.#teardownCountdown();
    this.#startPerformance();
  }

  #teardownCountdown(): void {
    if (this.countdownTimer !== undefined) clearTimeout(this.countdownTimer);
    this.countdownTimer = undefined;
  }

  // —— 内部：回放节拍 ——

  #startPerformance(): void {
    this.playing = true;
    this.#patch({ countdown: null });
    this.#scheduleBeat();
  }

  /**
   * 逐条消费队列：每类事件按 playbackMs 停留。
   * 节拍器只在空闲时排程（ingest 每次轮询都进来，重复排程会把同一条的等待时间不断重置饿死演出）。
   */
  #scheduleBeat(): void {
    if (this.destroyed) return;
    if (this.beatTimer !== undefined) return;
    if (this.snapshot.countdown !== null || !this.playing) return;
    const next = this.snapshot.events[this.snapshot.playhead + 1];
    if (!next) return;
    this.beatTimer = setTimeout(() => {
      this.beatTimer = undefined;
      this.#patch({ playhead: this.snapshot.playhead + 1 });
      this.#scheduleBeat();
    }, playbackMs(next));
  }

  /**
   * 当前事件的播报已完成（CaptionBar onAllSpoken 回调）：立即推进下一条。
   * 主路径——「说完才到下一个说」；playbackMs 定时器只是 TTS 失败的兑底。
   */
  speechDone(): void {
    if (this.destroyed || this.beatTimer === undefined) return;
    if (this.snapshot.countdown !== null || !this.playing) return;
    this.#teardownBeat();
    this.#patch({ playhead: this.snapshot.playhead + 1 });
    this.#scheduleBeat();
  }

  #teardownBeat(): void {
    if (this.beatTimer !== undefined) clearTimeout(this.beatTimer);
    this.beatTimer = undefined;
  }

  #patch(patch: Partial<WerewolfGameSnapshot<TStatus>>): void {
    this.snapshot = { ...this.snapshot, ...patch };
    for (const listener of this.listeners) listener(this.snapshot);
  }
}
