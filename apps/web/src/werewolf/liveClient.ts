/**
 * 真实对局客户端：server（/api/game/*）桥接。
 * - startGame：开局（板子 + 座位选手名，引擎自动推进）
 * - fetchGameStatus：轮询上帝视角事件流（含隐私事件，观战端自行取舍）
 * server 端点见 rust/server/src/routes/werewolf/mod.rs
 */

/** server 板子预设名（GameState::resolve_board） */
export type ServerBoardId = 'newbie_six' | 'nine_standard' | 'all_stars';

/** 前端板子 id → server 预设名 */
export function toServerBoard(id: string): ServerBoardId | undefined {
  const map: Record<string, ServerBoardId> = {
    rookie6: 'newbie_six',
    standard9: 'nine_standard',
    allstars12: 'all_stars',
  };
  return map[id];
}

export type GameEventDto = {
  seq: number;
  day: number;
  kind:
    | 'started'
    | 'night'
    | 'night_action'
    | 'wolf_chat'
    | 'wolf_pass'
    | 'dawn'
    | 'speech'
    | 'duel'
    | 'vote'
    | 'idiot'
    | 'hunter'
    | 'died'
    | 'over'
    | (string & {});
  seat: number | null;
  text: string;
};

export type GameStatusDto = {
  status: 'idle' | 'running' | 'paused' | 'ended' | (string & {});
  board: string;
  seed: number;
  day: number;
  phase: string;
  total: number;
  alive_total: number;
  roles: { name: string; count: number }[];
  winner: 'good' | 'werewolf' | null;
  ended_reason: 'settled' | 'manual' | null;
  seats: string[];
  events: GameEventDto[];
};

const BASE = '/api/game';

type ApiEnvelope<T> = { code: number; message: string; data: T };

function isEnvelope(value: unknown): value is ApiEnvelope<unknown> {
  return typeof value === 'object' && value !== null && 'code' in value && 'data' in value;
}

/** 关键字段守卫：server 契约的必填结构（其余字段信任 server） */
function isGameStatusDto(value: unknown): value is GameStatusDto {
  if (typeof value !== 'object' || value === null) return false;
  if (!('status' in value) || typeof value.status !== 'string') return false;
  if (!('seats' in value) || !Array.isArray(value.seats)) return false;
  if (!('events' in value) || !Array.isArray(value.events)) return false;
  if (!('day' in value) || typeof value.day !== 'number') return false;
  if (!('total' in value) || typeof value.total !== 'number') return false;
  if (!('alive_total' in value) || typeof value.alive_total !== 'number') return false;
  return true;
}

/** 拆开 ApiResponse 信封，返回原始 data（不做断言，由调用方类型守卫） */
async function unwrapData(res: Response): Promise<unknown> {
  const body: unknown = await res.json();
  if (!res.ok || !isEnvelope(body) || body.code !== 0) {
    const message = isEnvelope(body) ? body.message : `请求失败（HTTP ${res.status}）`;
    throw new Error(message);
  }
  return body.data;
}

function requireStatus(value: unknown): GameStatusDto {
  if (!isGameStatusDto(value)) throw new Error('server 返回数据格式异常');
  return { ...value, events: value.events ?? [] };
}

/** 开局：座位选手名由向导发牌结果传入；对局已在进行则直接复用（幂等观战） */
export async function startGame(board: ServerBoardId, players: string[]): Promise<GameStatusDto> {
  const res = await fetch(`${BASE}/start`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ board, players }),
  });
  return requireStatus(await unwrapData(res));
}

/** 当前对局状态与上帝视角事件流 */
export async function fetchGameStatus(): Promise<GameStatusDto> {
  const res = await fetch(`${BASE}/status`);
  return requireStatus(await unwrapData(res));
}
