/** 狼人杀观战页 —— 语音朗读开关、引擎切换与自动播报（页面级状态，组件保持纯展示） */

import { useCallback, useEffect, useRef, useState } from 'react';

import { fetchTtsStatus, speak, startTtsSetup, stopSpeaking, type TtsEngine } from './speech';

/** localStorage 键：值存引擎名（'local' | 'doubao'；旧版本值 '1' 视为 'local'） */
const STORAGE_KEY = 'werewolf-voice';

/** 语音开关状态：关 / 准备中（装模型或校验豆包）/ 开 / 失败可重试 */
export type VoicePhase = 'off' | 'preparing' | 'on' | 'failed';

export type UseVoiceResult = {
  /** 当前语音状态（按钮渲染依据） */
  phase: VoicePhase;
  /** 当前引擎（本地 kokoro / 豆包官方音色） */
  engine: TtsEngine;
  /** 切换引擎：关→只记偏好；开→立即切到新引擎（重新走启用流程） */
  setEngine: (engine: TtsEngine) => void;
  /** 点击开关按钮：关 → 校验/安装 → 开；开 → 关 */
  toggle: () => void;
};

function loadEngine(): TtsEngine {
  return localStorage.getItem(STORAGE_KEY) === 'doubao' ? 'doubao' : 'local';
}

function loadEnabled(): boolean {
  const saved = localStorage.getItem(STORAGE_KEY);
  return saved === '1' || saved === 'local' || saved === 'doubao';
}

/** 语音开关 hook：管理开关/引擎状态 + 模型安装轮询；自动播报见 useAutoSpeak */
export function useVoice(): UseVoiceResult {
  const [engine, setEngineState] = useState<TtsEngine>(loadEngine);
  const [phase, setPhase] = useState<VoicePhase>(() => (loadEnabled() ? 'preparing' : 'off'));

  const toggle = useCallback(() => {
    if (phase === 'on' || phase === 'preparing') {
      localStorage.removeItem(STORAGE_KEY);
      stopSpeaking();
      setPhase('off');
      return;
    }
    localStorage.setItem(STORAGE_KEY, engine);
    void enableVoice(engine, setPhase);
  }, [phase, engine]);

  const setEngine = useCallback(
    (next: TtsEngine) => {
      if (next === engine) return;
      setEngineState(next);
      stopSpeaking();
      if (phase === 'off') {
        // 未开启：只记偏好，下次开启生效
        localStorage.removeItem(STORAGE_KEY);
        return;
      }
      // 已开启：立即切引擎（豆包秒开/秒失败；本地则校验/安装模型）
      localStorage.setItem(STORAGE_KEY, next);
      setPhase('preparing');
      void enableVoice(next, setPhase);
    },
    [engine, phase],
  );

  // 上次开过开关 → 刷新后自动恢复（本地模型已装则秒开，豆包则校验 Cookie）
  useEffect(() => {
    if (loadEnabled()) {
      void enableVoice(loadEngine(), setPhase);
    }
  }, []);

  return { phase, engine, setEngine, toggle };
}

/** 校验并开启语音：豆包查 Cookie 可用性；本地查模型安装（未装则触发安装并轮询） */
async function enableVoice(
  engine: TtsEngine,
  setPhase: (phase: VoicePhase) => void,
): Promise<void> {
  setPhase('preparing');
  let status = await fetchTtsStatus().catch(() => null);
  if (!status) {
    setPhase('failed');
    return;
  }
  if (engine === 'doubao') {
    // 豆包云端引擎：无需模型，服务端配好 DOUBAO_COOKIE 即可用
    setPhase(status.doubao_available ? 'on' : 'failed');
    return;
  }
  if (!status.installed) {
    startTtsSetup();
    // 轮询等待安装完成（147MB，约 1-2 分钟）；刷新页面即中断，无残留副作用
    for (let waited = 0; waited < 180; waited += 5) {
      await new Promise((resolve) => setTimeout(resolve, 5000));
      status = await fetchTtsStatus().catch(() => null);
      if (!status || status.phase === 'failed') {
        setPhase('failed');
        return;
      }
      if (status.phase === 'ready') break;
    }
    if (status?.phase !== 'ready') {
      setPhase('failed');
      return;
    }
  }
  setPhase('on');
}

/**
 * 自动播报：语音开启后，主持人主持词与选手台词变化即朗读（主持人先说、选手接话，
 * speak 内部队列串行）。入参全是原始值，避免对象引用每帧变化导致重复触发。
 */
export function useAutoSpeak(
  phase: VoicePhase,
  engine: TtsEngine,
  hostMessage?: string,
  speakerName?: string,
  speakerText?: string,
): void {
  const enabled = phase === 'on';
  const spokenRef = useRef<string>('');

  useEffect(() => {
    if (!enabled) return;
    const lines: Array<{ voice: string; text: string }> = [];
    if (hostMessage) lines.push({ voice: '主持人', text: hostMessage });
    if (speakerName && speakerText) lines.push({ voice: speakerName, text: speakerText });
    const key = lines.map((line) => `${line.voice}:${line.text}`).join('|');
    if (key === '' || key === spokenRef.current) return;
    spokenRef.current = key;
    for (const line of lines) {
      void speak(line.voice, line.text, engine).catch(() => undefined);
    }
  }, [enabled, engine, hostMessage, speakerName, speakerText]);
}
