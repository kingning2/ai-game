import { Card, Spinner, cn } from '@agent-packages/shared-ui';
import { useEffect, useState } from 'react';
import { compactGameMemory, fetchGameMemory, type GameMemory } from './memory';

export type MemoryPanelProps = {
  /** 座位总数（9 人局 0..8） */
  seats: number;
  onClose: () => void;
  className?: string;
};

/** 标签语义色：狼聊红、摘要紫、其余蓝灰 */
function tagTone(tag: string): string {
  if (tag === 'wolfchat') return 'bg-red/10 text-red';
  if (tag === 'summary') return 'bg-purple/10 text-purple';
  return 'bg-blue/10 text-blue';
}

/**
 * 玩家记忆面板：观战「某号眼里的对局」——条目流水 + LLM 压缩。
 * 压缩是显式动作（划分 → LLM 摘要 → 落回），未超窗时按钮返回"无需压缩"。
 */
export function MemoryPanel({ seats, onClose, className }: MemoryPanelProps) {
  const [seat, setSeat] = useState(0);
  const [memory, setMemory] = useState<GameMemory | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [compacting, setCompacting] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const data = await fetchGameMemory(seat);
        if (!cancelled) {
          setMemory(data);
          setError(null);
        }
      } catch (err: unknown) {
        if (!cancelled) {
          setMemory(null);
          setError(err instanceof Error ? err.message : '记忆加载失败');
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    void load();
    return () => {
      cancelled = true;
    };
  }, [seat]);

  async function handleCompact() {
    setCompacting(true);
    setNotice(null);
    setError(null);
    try {
      const result = await compactGameMemory(seat);
      setNotice(
        result.compacted
          ? `已压缩为 ${result.entries} 条（摘要：${result.summary ?? ''}）`
          : `仅 ${result.entries} 条，未超窗，无需压缩`,
      );
      setMemory(await fetchGameMemory(seat));
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : '压缩失败');
    } finally {
      setCompacting(false);
    }
  }

  return (
    <Card className={cn('w-80', className)}>
      <div className="flex items-center justify-between">
        <span className="text-sm font-semibold text-label">玩家记忆</span>
        <button
          type="button"
          onClick={onClose}
          className="rounded-full px-2 py-0.5 text-xs text-secondary-label hover:bg-fill hover:text-label"
        >
          ✕
        </button>
      </div>

      {/* 座位切换 */}
      <div className="mt-2 flex flex-wrap gap-1">
        {Array.from({ length: seats }, (_, i) => (
          <button
            key={i}
            type="button"
            onClick={() => setSeat(i)}
            className={cn(
              'h-6 w-6 rounded-full text-xs font-medium transition-colors',
              i === seat
                ? 'bg-blue text-white'
                : 'bg-fill text-secondary-label hover:bg-blue/10 hover:text-blue',
            )}
          >
            {i + 1}
          </button>
        ))}
      </div>

      {loading && (
        <div className="mt-3 flex items-center justify-center gap-2 text-xs text-secondary-label">
          <Spinner size="sm" /> 加载中…
        </div>
      )}
      {error && <p className="mt-3 text-xs text-red">{error}</p>}

      {memory && !loading && (
        <>
          <div className="mt-3 max-h-64 overflow-y-auto">
            {memory.entries.length === 0 ? (
              <p className="py-4 text-center text-xs text-secondary-label">
                {memory.player} 号还没有可记的事实（第一夜尚未开始）
              </p>
            ) : (
              <ul className="flex flex-col gap-1.5">
                {memory.entries.map((entry) => (
                  <li
                    key={`${entry.seq}-${entry.tag}-${entry.text}`}
                    className="flex items-start gap-1.5 text-xs"
                  >
                    <span
                      className={cn(
                        'shrink-0 rounded px-1 py-0.5 text-[10px] font-medium',
                        tagTone(entry.tag),
                      )}
                    >
                      {entry.tag}
                    </span>
                    <span className="leading-relaxed text-label">{entry.text}</span>
                  </li>
                ))}
              </ul>
            )}
          </div>
          <div className="mt-3 flex items-center justify-between">
            <span className="text-xs text-secondary-label">{memory.entries.length} 条</span>
            <button
              type="button"
              onClick={() => {
                void handleCompact();
              }}
              disabled={compacting}
              className={cn(
                'rounded-full bg-blue px-3 py-1 text-xs font-medium text-white',
                'transition-opacity hover:opacity-90 disabled:opacity-50',
              )}
            >
              {compacting ? '压缩中…' : '🤖 LLM 压缩'}
            </button>
          </div>
          {notice && <p className="mt-2 text-xs leading-relaxed text-secondary-label">{notice}</p>}
        </>
      )}
    </Card>
  );
}
