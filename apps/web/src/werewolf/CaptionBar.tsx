import { useEffect, useMemo, useState } from 'react';
import { cn } from '@agent-packages/shared-ui';
import { speak, startTtsSetup, stopSpeaking } from './speech';

/** 按中英标点切句（。！？!?…；;），保留标点；空段过滤 */
function splitSentences(text: string): string[] {
  return (text.match(/[^。！？!?…;;]+[。！？!?…;;]*/g) ?? []).map((s) => s.trim()).filter(Boolean);
}

/** 播报队列的一句：文本 + 该句用什么音色 */
type QueueItem = { sentence: string; voice: string };

/**
 * 底部电影字幕条：把当前念白按句切分逐句呈现（一句一句出现，不是整段）。
 *
 * 传 `preamble` 时（如选手发言前的「3 号玩家请发言」），主持人音色先播报完
 * 再进入正文；正文按 `voice` 逐句朗读——语音播完才推下一句，字幕与语音同步；
 * 合成失败或未装模型时自动降级为按句长估算的定时推进。
 * 念白切换时打断旧播报、重置回第一句。
 */
export function CaptionBar({
  text,
  voice,
  preamble,
  className,
}: {
  /** 当前念白全文（主持人词或选手发言） */
  text: string;
  /** 朗读音色：音色 id / `主持人` / 选手名；省略则纯字幕不发声 */
  voice?: string;
  /** 正文前的主持报号（用主持人音色播完才进正文，如「一号玩家请发言」） */
  preamble?: string;
  className?: string;
}) {
  const queue = useMemo<QueueItem[]>(() => {
    const lead = (preamble ? splitSentences(preamble) : []).map((sentence) => ({
      sentence,
      voice: '主持人',
    }));
    return [...lead, ...splitSentences(text).map((sentence) => ({ sentence, voice: voice ?? '' }))];
  }, [text, voice, preamble]);
  const [index, setIndex] = useState(0);
  const [prevKey, setPrevKey] = useState(text + (preamble ?? ''));

  // 念白/报号切换：渲染期重置回第一句（adjusting-state-when-props-change 官方模式）
  const key = text + (preamble ?? '');
  if (prevKey !== key) {
    setPrevKey(key);
    setIndex(0);
  }

  // 逐句推进：有音色时以「该句播完」为推进信号（合成失败降级定时器）；
  // 无音色按句长自适应估算（短句 1.8s，长句 90ms/字，上限 5s）
  useEffect(() => {
    const item = queue[index];
    if (!item) return undefined;
    if (index >= queue.length - 1 && queue.length <= 1) return undefined;
    const advance = (): void => setIndex((i) => i + 1);
    if (!item.voice || !item.sentence) {
      const dur = Math.min(5000, Math.max(1800, (item.sentence?.length ?? 0) * 90));
      const timer = setTimeout(advance, dur);
      return () => clearTimeout(timer);
    }
    startTtsSetup(); // 幂等：未装模型时后台补装，已装立即返回
    let fallback: ReturnType<typeof setTimeout> | undefined;
    speak(item.voice, item.sentence)
      .then(advance)
      .catch(() => {
        const dur = Math.min(5000, Math.max(1800, item.sentence.length * 90));
        fallback = setTimeout(advance, dur);
      });
    return () => {
      if (fallback) clearTimeout(fallback);
      // 正常推进时旧句已播完，清空的是空队列；念白切换时则打断旧播报
      stopSpeaking();
    };
  }, [index, queue]);

  if (queue.length === 0) return null;
  const current = queue[Math.min(index, queue.length - 1)];
  return (
    <div
      className={cn(
        'mx-auto max-w-2xl rounded-xl bg-arena/55 px-5 py-2.5 text-center backdrop-blur-sm',
        className,
      )}
    >
      {current.voice === '主持人' && voice && voice !== '主持人' ? (
        <p className="mb-0.5 text-xs font-semibold tracking-wider text-arena-gold/90">主持人报号</p>
      ) : null}
      <p
        key={index}
        className="animate-fade-in text-base leading-relaxed font-bold text-arena-text drop-shadow-md"
      >
        {current.sentence}
      </p>
      {queue.length > 1 ? (
        <p className="mt-0.5 text-xs text-arena-muted">
          {Math.min(index + 1, queue.length)} / {queue.length}
        </p>
      ) : null}
    </div>
  );
}
