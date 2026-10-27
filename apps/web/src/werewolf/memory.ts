/** 狼人杀观战页 —— 玩家记忆客户端（对接 Rust 伴生服务 /api/game/memory） */

/** 单条对局记忆（通用记忆模型 + 对局领域标签） */
export type MemoryEntry = {
  /** 对局天（时间片） */
  seq: number;
  /** 领域标签：speech / wolfchat / dawn / vote / seercheck / summary */
  tag: string;
  /** 记录者座位（系统播报为 null） */
  source: string | null;
  /** 事实内容 */
  text: string;
};

/** GET /api/game/memory 响应：某玩家视角的对局记忆（决策 prompt 事实段） */
export type GameMemory = {
  player: string;
  entries: MemoryEntry[];
  rendered: string;
};

/** POST /api/game/memory/compact 响应 */
export type MemoryCompactResult = {
  /** false = 未超窗，无需压缩，模型未被调用 */
  compacted: boolean;
  entries: number;
  /** LLM 摘要正文（compacted = false 时为 null） */
  summary: string | null;
};

/** 统一响应包裹：code === 0 表示成功 */
type ApiEnvelope<T> = { code: number; message: string; data: T };

function isEnvelope<T>(value: unknown): value is ApiEnvelope<T> {
  return (
    typeof value === 'object' &&
    value !== null &&
    'code' in value &&
    'message' in value &&
    'data' in value
  );
}

async function unwrap<T>(response: Response): Promise<T> {
  const raw: unknown = await response.json();
  if (!isEnvelope<T>(raw)) throw new Error('响应不是合法的 ApiResponse 包裹');
  if (raw.code !== 0) throw new Error(raw.message);
  return raw.data;
}

/** 查询 N 号玩家视角的对局记忆（做过 LLM 压缩则含摘要行） */
export async function fetchGameMemory(player: number): Promise<GameMemory> {
  return unwrap<GameMemory>(await fetch(`/api/game/memory?player=${player}`));
}

/** 调 LLM 压缩记忆：老段摘要 + 最近 keepRecent 条保留原文；未超窗返回 compacted=false */
export async function compactGameMemory(
  player: number,
  keepRecent?: number,
): Promise<MemoryCompactResult> {
  return unwrap<MemoryCompactResult>(
    await fetch('/api/game/memory/compact', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ player, keep_recent: keepRecent }),
    }),
  );
}
