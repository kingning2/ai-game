import { useCallback, useEffect, useRef, useState } from 'react';

export type RecorderState = 'idle' | 'recording' | 'error';

/** 对局录像文件名前缀的安全化（去路径分隔与空白） */
function safeLabel(label: string): string {
  return label.replace(/[\\/:*?"<>|\s]+/g, '-').slice(0, 40) || '对局';
}

function fmt(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

/**
 * 对局录制：getDisplayMedia 抓「当前标签页」的画面 + 声音（含 TTS 播报），
 * MediaRecorder 编码 webm，停止（手动或终局自动）后自动触发下载。
 *
 * 浏览器会弹共享授权框——用户需选择「当前标签页」并勾选「同时共享标签页声音」，
 * 这样录制件里才有语音播报。用户在浏览器 UI 点「停止共享」同样会收束到停止态。
 */
export function useRecorder(label: string) {
  const recRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const [state, setState] = useState<RecorderState>('idle');
  const [seconds, setSeconds] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const start = useCallback(async (): Promise<void> => {
    if (recRef.current?.state === 'recording') return;
    try {
      const stream = await navigator.mediaDevices.getDisplayMedia({
        video: { frameRate: 30 },
        audio: true,
      });
      const mime =
        ['video/webm;codecs=vp9,opus', 'video/webm;codecs=vp8,opus', 'video/webm'].find((m) =>
          MediaRecorder.isTypeSupported(m),
        ) ?? 'video/webm';
      const rec = new MediaRecorder(stream, { mimeType: mime, videoBitsPerSecond: 4_000_000 });
      chunksRef.current = [];
      rec.ondataavailable = (e: BlobEvent) => {
        if (e.data.size > 0) chunksRef.current.push(e.data);
      };
      rec.onstop = () => {
        stream.getTracks().forEach((t) => t.stop());
        const blob = new Blob(chunksRef.current, { type: mime });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${safeLabel(label)}-${new Date().toISOString().slice(0, 19).replaceAll(':', '-')}.webm`;
        a.click();
        URL.revokeObjectURL(url);
        recRef.current = null;
        setState('idle');
        setSeconds(0);
      };
      // 用户在浏览器条上点「停止共享」：track ended → 收束为一次正常停止
      stream.getVideoTracks()[0]?.addEventListener('ended', () => {
        if (rec.state !== 'inactive') rec.stop();
      });
      rec.start(1000); // 每秒落一个 chunk，中途崩溃也能保留已录内容
      recRef.current = rec;
      setState('recording');
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : '无法开始录制（需要标签页共享授权）');
      setState('error');
    }
  }, [label]);

  const stop = useCallback((): void => {
    if (recRef.current?.state === 'recording') recRef.current.stop();
  }, []);

  // 录制计时
  useEffect(() => {
    if (state !== 'recording') return undefined;
    const timer = setInterval(() => setSeconds((s) => s + 1), 1000);
    return () => clearInterval(timer);
  }, [state]);

  return { state, seconds, error, start, stop, recording: state === 'recording' };
}

/** 录制控制按钮：idle 显示「开始录制」，recording 显示时长 +「停止并保存」 */
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
        <span className="size-2 animate-pulse rounded-full bg-white" />⏹ {fmt(seconds)} 停止并保存
      </button>
    );
  }
  return (
    <button
      type="button"
      onClick={onStart}
      title={error ?? '录制当前标签页画面与声音，停止后自动下载 webm'}
      className="rounded-full bg-arena/50 px-3 py-1.5 text-xs font-medium text-arena-text shadow-card backdrop-blur-sm transition-colors hover:bg-arena/70"
    >
      🔴 录制对局
    </button>
  );
}
