/**
 * server（/api/*）桥接 —— web 侧对接后端的全部业务都在这里：
 * 端点、DTO、定位参数归 web；@agent-packages/game 的 WerewolfGame 基类只收 ingest 喂入的数据。
 * 请求统一走 @agent-packages/shared 的 createFetchClient（信封解包 / 超时 / 幂等去重内置），
 * 禁止裸 fetch。server 端点见 rust/server/src/routes/werewolf/mod.rs
 */
import { createFetchClient, isArray, isObject } from '@agent-packages/shared';
import type { GameEventDto } from '@agent-packages/game';

export type { GameEventDto };

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
  /** 座位 → 角色显示名（上帝视角，与引擎真实发牌对齐） */
  seat_roles?: string[];
  /** 服务器重启后的中断局（降级结算，真实身份从落库反推） */
  stale?: boolean;
  /** 赛后 MVP 投票：null = 无 MVP 流程（手动终局 / stale 中断局） */
  mvp?: MvpDto | null;
  events: GameEventDto[];
};

/** 单张 MVP 选票 */
export type MvpVoteDto = {
  voter: string;
  candidate: string;
  reason: string;
};

/** 赛后 MVP 投票状态：pending 投票中（票从 server 轮询）；done 已决出 */
export type MvpDto = {
  phase: 'pending' | 'done' | (string & {});
  winner: string | null;
  votes: MvpVoteDto[];
};

/** 房间信息（SQLite 落库） */
export type RoomInfo = {
  code: string;
  name: string;
  board_id: string;
  status: 'waiting' | 'playing' | 'finished';
  created_at: string;
};

/** 排行榜行：跨对局聚合（全部 finished 局 × 房间选手） */
export type LeaderboardRowDto = {
  name: string;
  games: number;
  wins: number;
  mvp_count: number;
  /** 胜率 0..1 */
  win_rate: number;
};

/** 对局定位参数：?room=房间码 优先，或 ?game=对局 id；都缺省用 server 最近一局 */
export type GameLocator = { room?: string; game?: number };

const client = createFetchClient({ baseURL: '/api', timeout: 8000 });

/** events 缺省兜底为空数组（server 旧版本可能不带事件流） */
function normalizeStatus(status: GameStatusDto): GameStatusDto {
  return { ...status, events: status.events ?? [] };
}

/** 开局：座位选手名由向导发牌结果传入；对局已在进行则直接复用（幂等观战） */
export async function startGame(
  board: ServerBoardId,
  players: string[],
  roomCode?: string,
): Promise<GameStatusDto> {
  const body = roomCode ? { board, players, room_code: roomCode } : { board, players };
  return normalizeStatus(await client.post<GameStatusDto>('/game/start', body));
}

/** 当前对局状态与上帝视角事件流（多槽位：带 room 定位到该房间的对局） */
export async function fetchGameStatus(locator?: GameLocator): Promise<GameStatusDto> {
  return normalizeStatus(
    await client.get<GameStatusDto>('/game/status', {
      query: { room: locator?.room, game: locator?.game },
    }),
  );
}

/** 创建房间（真实落库；返回房间号） */
export async function createRoom(board: ServerBoardId, players: string[]): Promise<RoomInfo> {
  return client.post<RoomInfo>('/rooms', { board_id: board, players });
}

/** 最近房间列表（新→旧，上限 50） */
export async function fetchRooms(): Promise<RoomInfo[]> {
  const data = await client.get<RoomInfo[]>('/rooms');
  if (!isArray(data) || !data.every(isObject)) throw new Error('房间列表格式异常');
  return data;
}

/** 胜利排行榜（server 聚合，按胜场→胜率→场次排序） */
export async function fetchLeaderboard(): Promise<LeaderboardRowDto[]> {
  const data = await client.get<LeaderboardRowDto[]>('/leaderboard');
  if (!isArray(data) || !data.every(isObject)) throw new Error('排行榜数据格式异常');
  return data;
}
