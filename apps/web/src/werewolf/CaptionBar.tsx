import { useEffect, useMemo, useRef, useState } from 'react';
import { cn } from '@agent-packages/shared-ui';
import { prefetch, speak, startTtsSetup, stopSpeaking } from './speech';

/** 按中英标点切句（。！？!?…；;），保留标点；空段过滤 */
function splitSentences(text: string): string[] {
  return (text.match(/[^。！？!?…;;]+[。！？!?…;;]*/g) ?? []).map((s) => s.trim()).filter(Boolean);
}

/** 播报队列的一句：文本 + 该句用什么音色 */
type QueueItem = { sentence: string; voice: string };

/**
 * 底部电影字幕条：只显示当前正在朗读的一句（整段全文由上方发言面板负责），
 * 超长单行截断——观众视线跟字幕走，不重复信息。
 *
 * 语音按句逐句朗读，播完一句立刻推进下一句；开播时预取下一句音频，
 * 消除句间合成等待。合成失败或未装模型时自动降级为按句长估算的定时推进。
 * 念白切换时打断旧播报、重置回第一句。
 */
export function CaptionBar({
  text,
  voice,
  segments,
  voices,
  onAllSpoken,
  className,
}: {
  /** 当前念白全文（主持人词或选手发言）；`segments` 存在时忽略 */
  text: string;
  /** 朗读音色：音色 id / `主持人` / 选手名；省则纯字幕不发声 */
  voice?: string;
  /** 多人多段台词（如夜晚狼聊）：按段序依次用各自的音色播报 */
  segments?: { voice: string; text: string }[];
  /** 选手名/「主持人」→ Edge 音色 id 映射；提供时走 Edge 引擎（发音更好听），
   *  查不到的名字回退本地引擎（历史局 / 未分配音色） */
  voices?: Record<string, string>;
  /** 队列全部播完（含降级）时回调一次：用于「说完才推进下一条事件」 */
  onAllSpoken?: () => void;
  className?: string;
}) {
  const queue = useMemo<QueueItem[]>(
    () =>
      segments
        ? segments.flatMap((seg) =>
            splitSentences(seg.text).map((sentence) => ({ sentence, voice: seg.voice })),
          )
        : splitSentences(text).map((sentence) => ({ sentence, voice: voice ?? '' })),
    [text, voice, segments],
  );
  const [index, setIndex] = useState(0);
  const [prevKey, setPrevKey] = useState(text);
  /** 全部句子播完（含降级定时器路径）回调：驱动对局节拍器推进下一条事件 */
  const onAllSpokenRef = useRef(onAllSpoken);
  useEffect(() => {
    onAllSpokenRef.current = onAllSpoken;
  });

  // 念白切换：渲染期重置回第一句（adjusting-state-when-props-change 官方模式）
  const segKey = segments?.map((s) => `${s.voice}:${s.text}`).join('|') ?? '';
  const key = text + segKey;
  if (prevKey !== key) {
    setPrevKey(key);
    setIndex(0);
  }

  // 逐句推进：有音色时以「该句播完」为推进信号（合成失败降级定时器）；
  // 无音色按句长自适应估算（短句 1.8s，长句 90ms/字，上限 5s）。
  // 最后一句播完（含降级）→ onAllSpoken：对局节拍器「说完才演下一条」的驱动源
  useEffect(() => {
    if (queue.length === 0) return undefined;
    const item = queue[index];
    if (!item) {
      // 多句队列播完（index 越界）：通知一次即可
      onAllSpokenRef.current?.();
      return undefined;
    }
    // 开播即预取下一句音频：播完当前句零等待接下一句
    const next = queue[index + 1];
    if (next !== undefined) {
      const nextEdge = voices?.[next.voice];
      if (nextEdge !== undefined) prefetch(nextEdge, next.sentence, 'edge');
      else if (next.voice) prefetch(next.voice, next.sentence);
    }
    let cancelled = false;
    const finish = (): void => {
      if (cancelled) return;
      if (index >= queue.length - 1) onAllSpokenRef.current?.();
      else setIndex((i) => i + 1);
    };
    let fallback: ReturnType<typeof setTimeout> | undefined;
    const downgrade = (): void => {
      const dur = Math.min(5000, Math.max(1800, item.sentence.length * 90));
      fallback = setTimeout(finish, dur);
    };
    const edgeVoice = voices?.[item.voice];
    if (!item.voice || !item.sentence) {
      downgrade();
    } else if (edgeVoice) {
      speak(edgeVoice, item.sentence, 'edge').then(finish).catch(downgrade);
    } else {
      startTtsSetup(); // 幂等：未装模型时后台补装，已装立即返回
      speak(item.voice, item.sentence).then(finish).catch(downgrade);
    }
    return () => {
      cancelled = true;
      if (fallback) clearTimeout(fallback);
      // 正常推进时旧句已播完，清空的是空队列；念白切换时则打断旧播报
      stopSpeaking();
    };
  }, [index, queue, voices]);

  if (queue.length === 0) return null;
  return (
    <div
      className={cn(
        'mx-auto max-w-2xl rounded-xl bg-arena/55 px-5 py-3 text-center backdrop-blur-sm',
        className,
      )}
    >
      <p
        key={index}
        className="animate-fade-in truncate text-lg leading-relaxed font-medium text-arena-text"
      >
        {queue[index]?.sentence}
      </p>
    </div>
  );
}
