import { useCallback, useSyncExternalStore } from 'react';
import { openWsStream, type WsStreamHandle } from '@agent-packages/shared';

export type RecorderState = 'idle' | 'connecting' | 'recording' | 'saved' | 'error';

/**
 * 对局录制（推流版）：getDisplayMedia 抓当前标签页 → MediaRecorder 出 webm chunk →
 * WebSocket 推给 server → server 转 ffmpeg -c copy 落盘 recordings/<room>-<ts>.webm。
 * 文件保存在 server 侧，前端不再下载。
 *
 * 模块级单例：跨页面存活——介绍页（RoomSetup）自动启动，跳到对局页继续录，终局自动停。
 * 浏览器安全模型要求 getDisplayMedia 必须弹一次授权（选「当前标签页」+ 勾选共享声音，
 * 录制件里才有 TTS 播报）；除此之外全自动。授权失败/ffmpeg 不可用 → error 态，不阻塞对局。
 */
let rec: MediaRecorder | null = null;
let stream: MediaStream | null = null;
let ws: WsStreamHandle | null = null;
let chunks: Blob[] = [];
let state: RecorderState = 'idle';
let seconds = 0;
let error: string | null = null;
let timer: ReturnType<typeof setInterval> | null = null;

type Listener = () => void;
const listeners = new Set<Listener>();

// 快照缓存：useSyncExternalStore 的 getSnapshot 必须返回稳定引用，
// 否则 React 渲染期比对不等会无限重渲染（Maximum update depth exceeded）
let snap: RecorderSnapshot = { state, seconds, error };

function emit(): void {
  snap = { state, seconds, error };
  for (const listener of listeners) listener();
}

function snapshot(): RecorderSnapshot {
  return snap;
}

export interface RecorderSnapshot {
  state: RecorderState;
  seconds: number;
  error: string | null;
}

function setState(next: RecorderState, message?: string): void {
  state = next;
  error = message ?? null;
  emit();
}

function startTimer(): void {
  stopTimer();
  timer = setInterval(() => {
    seconds += 1;
    emit();
  }, 1000);
}

function stopTimer(): void {
  if (timer !== null) {
    clearInterval(timer);
    timer = null;
  }
}

/** 收束一次会话：停计时、关流、释放轨道、回 idle（server 侧 ffmpeg 收尾出完整文件） */
function teardown(): void {
  stopTimer();
  ws?.close();
  ws = null;
  stream?.getTracks().forEach((t) => t.stop());
  stream = null;
  rec = null;
  chunks = [];
  seconds = 0;
}

/**
 * 开始录制（幂等）：已在录（跨页面接管）直接返回；connecting 中忽略重复调用。
 * @param label 显示名（server 文件名主体）
 * @param room 房间号（ws 查询参数，server 关联对局）
 */
async function begin(label: string, room?: string): Promise<void> {
  if (rec?.state === 'recording' || state === 'connecting') return;
  setState('connecting');
  try {
    stream = await navigator.mediaDevices.getDisplayMedia({
      video: { frameRate: 30 },
      audio: true,
    });
  } catch (e) {
    setState('error', e instanceof Error ? e.message : '标签页共享被取消，本次不录制');
    return;
  }

  const mime =
    ['video/webm;codecs=vp9,opus', 'video/webm;codecs=vp8,opus', 'video/webm'].find((m) =>
      MediaRecorder.isTypeSupported(m),
    ) ?? 'video/webm';
  const recorder = new MediaRecorder(stream, { mimeType: mime, videoBitsPerSecond: 4_000_000 });
  chunks = [];
  rec = recorder;

  ws = openWsStream({
    path: '/api/game/record',
    query: { room: room ?? label, label },
    // server ffmpeg 启动失败 / 落盘完成回执；saved 时保持 recording 直到本地 stop
    onMessage: (data) => {
      try {
        const msg = JSON.parse(data) as { type: string; message?: string };
        if (msg.type === 'error') setState('error', msg.message ?? '服务端录制失败');
        if (msg.type === 'saved') {
          // 文件已落盘（server recordings/），收尾关连接
          ws?.close();
          ws = null;
          seconds = 0;
          setState('saved');
        }
      } catch {
        // 非约定帧忽略
      }
    },
    // 连接断了（主动 close 之外）= server 录制中断（ffmpeg 崩/服务重启），收束本地会话
    onClose: () => {
      if (state === 'recording' || state === 'connecting') {
        teardown();
        setState('idle');
      }
    },
  });

  recorder.ondataavailable = (e: BlobEvent) => {
    if (e.data.size > 0) {
      chunks.push(e.data);
      ws?.send(e.data);
    }
  };
  recorder.onstop = () => {
    chunks = [];
    stopTimer();
    stream?.getTracks().forEach((t) => t.stop());
    stream = null;
    rec = null;
    // 连接保持：先发 stop 控制帧（server 收到即落盘并回 saved），收到回执再关连接
    setState('recording'); // 保持状态，避免闪回 idle
    ws?.send('{"type":"stop"}');
  };
  // 用户在浏览器条上点「停止共享」：track ended → 视为一次正常停止
  stream.getVideoTracks()[0]?.addEventListener('ended', () => {
    if (recorder.state === 'recording') recorder.stop();
  });

  recorder.start(1000); // 每秒一个 chunk，逐帧推流（server 断电也只丢最后一秒）
  startTimer();
  setState('recording');
}

/** 结束录制：发 stop 控制帧让 server 收尾落盘，等 saved 回执后关连接 */
function stop(): void {
  if (rec?.state === 'recording') {
    rec.stop(); // onstop 收尾（发 stop 帧、释放轨道、等 saved）
    return;
  }
  teardown();
  setState('idle');
}

export function useRecorder() {
  const snap = useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    snapshot,
    snapshot,
  );
  const beginOnce = useCallback((label: string, room?: string) => begin(label, room), []);
  const stopOnce = useCallback(() => stop(), []);
  return {
    ...snap,
    start: beginOnce,
    stop: stopOnce,
    recording: snap.state === 'recording' || snap.state === 'connecting',
  };
}
function fmt(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

/** 录制控制按钮：recording 显示时长 +「结束录制」；idle 显示「开始录制」 */
export function RecordButton({
  recording,
  seconds,
  error,
  onStart,
  onStop,
}: {
  recording: boolean;
  seconds: number;
  error: string | null;
  onStart: () => void;
  onStop: () => void;
}) {
  if (recording) {
    return (
      <button
        type="button"
        onClick={onStop}
        className="flex items-center gap-1.5 rounded-full bg-red/90 px-3 py-1.5 text-xs font-bold text-arena-text shadow-card transition-colors hover:bg-red"
      >
        <span className="size-2 animate-pulse rounded-full bg-white" />⏹ {fmt(seconds)} 结束录制
      </button>
    );
  }
  return (
    <button
      type="button"
      onClick={onStart}
      title={error ?? '录制当前标签页画面与声音，推流到服务端保存为 webm'}
      className="rounded-full bg-arena/50 px-3 py-1.5 text-xs font-medium text-arena-text shadow-card backdrop-blur-sm transition-colors hover:bg-arena/70"
    >
      🔴 录制对局
    </button>
  );
}
