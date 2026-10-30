/** 狼人杀观战页 —— 本地语音合成客户端（对接 Rust 伴生服务 /api/tts） */

/** 语音引擎：本地 kokoro（离线，WAV）/ 豆包官方音色（云端，需服务端配 Cookie）/ Edge 免费接口（云端，MP3，无需 Key） */
export type TtsEngine = 'local' | 'doubao' | 'edge';

/** 服务端 TTS 装配状态（GET /api/tts/status） */
export type TtsStatus = {
  installed: boolean;
  /** idle / installing / ready / failed */
  phase: 'idle' | 'installing' | 'ready' | 'failed';
  engine_loaded: boolean;
  cache_dir: string;
  /** 豆包云端引擎是否可用（服务端已配置 DOUBAO_COOKIE） */
  doubao_available: boolean;
  voices: { id: string; label: string }[];
};

/** 统一响应包裹：code === 0 表示成功 */
type ApiEnvelope<T> = { code: number; message: string; data: T };

async function unwrap<T>(response: Response): Promise<T> {
  const body = (await response.json()) as ApiEnvelope<T>;
  if (body.code !== 0) throw new Error(body.message);
  return body.data;
}

/** 查询装配状态（模型是否已装、引擎是否已加载、音色表） */
export async function fetchTtsStatus(): Promise<TtsStatus> {
  return unwrap<TtsStatus>(await fetch('/api/tts/status'));
}

/** 触发后台安装模型（约 147MB，用 fetchTtsStatus 轮询 phase 直到 ready/failed）；幂等 */
export function startTtsSetup(): void {
  void fetch('/api/tts/setup', { method: 'POST' });
}

/** 串行播放队列：连续多条台词排队依次播，不互相打断 */
let queue: Promise<void> = Promise.resolve();
let current: HTMLAudioElement | null = null;

/** 停止当前播放并清空队列（关开关 / 页面卸载时用） */
export function stopSpeaking(): void {
  current?.pause();
  current = null;
  queue = Promise.resolve();
}

/**
 * 用指定音色朗读一句台词，返回播放完成的 Promise。
 *
 * @param voice  音色 id（`zf_001`）或选手名（`豆包`）/ `主持人`；豆包引擎传音色简称（`taozi`）
 * @param text   台词（≤500 字符，服务端校验）
 * @param engine 省略走本地 kokoro；`doubao` 走豆包官方云端音色
 */
export function speak(voice: string, text: string, engine?: TtsEngine): Promise<void> {
  const play = async (): Promise<void> => {
    const response = await fetch('/api/tts/speak', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(engine ? { voice, text, engine } : { voice, text }),
    });
    if (!response.ok) throw new Error(`TTS 合成失败（HTTP ${response.status}）`);
    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    const audio = new Audio(url);
    current = audio;
    audio.play().catch(() => undefined);
    await new Promise<void>((resolve) => {
      const done = (): void => resolve();
      audio.onended = done;
      audio.onerror = done;
    });
    URL.revokeObjectURL(url);
    if (current === audio) current = null;
  };
  const turn = queue.then(play, play);
  // 单句失败不拖垮后续排队
  queue = turn.catch(() => undefined);
  return turn;
}
