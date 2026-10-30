/**
 * werewolf 域端点函数 —— /api/* 的全部调用收口在这里，一函数一端点，返回已解包的业务数据
 * （shared 的 createFetchClient 自带 ApiResponse 信封解包，无需再 .then((r) => r.data)）。
 * 请求通道（http 实例 / 故障分类）见 ./http.ts，DTO 契约见 ./types.ts。
 * SSE 实时流（流式响应 createFetchClient 不覆盖）也归本文件。
 */
import { createSseStream, isArray, isObject } from '@agent-packages/shared';
import { http } from './http';
import type {
  GameLocator,
  GameStatusDto,
  LeaderboardRowDto,
  RoomInfo,
  ScriptDto,
  ScriptId,
  StreamEnd,
} from './types';

/** events 缺省兜底为空数组（server 旧版本可能不带事件流） */
function normalizeStatus(status: GameStatusDto): GameStatusDto {
  return { ...status, events: status.events ?? [] };
}

/** 开局：座位选手名由向导发牌结果传入；对局已在进行则直接复用（幂等观战）。board 即剧本 id */
export async function startGame(
  board: ScriptId,
  players: string[],
  roomCode?: string,
): Promise<GameStatusDto> {
  if (roomCode !== undefined) {
    return normalizeStatus(
      await http.post<GameStatusDto>('/game/start', { board, players, room_code: roomCode }),
    );
  }
  return normalizeStatus(await http.post<GameStatusDto>('/game/start', { board, players }));
}

/** 内置剧本清单：玩法选择卡与介绍页的单一事实源（一个剧本 = 一种玩法） */
export async function fetchScripts(): Promise<ScriptDto[]> {
  const data = await http.get<ScriptDto[]>('/scripts');
  if (!isArray(data) || !data.every(isObject)) throw new Error('剧本列表格式异常');
  return data;
}

/** 当前对局状态与上帝视角事件流（多槽位：带 room 定位到该房间的对局） */
export async function fetchGameStatus(locator?: GameLocator): Promise<GameStatusDto> {
  return normalizeStatus(
    await http.get<GameStatusDto>('/game/status', {
      query: { room: locator?.room, game: locator?.game },
    }),
  );
}

/** 创建房间（真实落库；返回房间号）。board_id 即剧本 id（server 剧本名 = 静态板子 id） */
export const createRoom = (board: string, players: string[]) =>
  http.post<RoomInfo>('/rooms', { board_id: board, players });

/** 最近房间列表（新→旧，上限 50） */
export async function fetchRooms(): Promise<RoomInfo[]> {
  const data = await http.get<RoomInfo[]>('/rooms');
  if (!isArray(data) || !data.every(isObject)) throw new Error('房间列表格式异常');
  return data;
}

/** 胜利排行榜（server 聚合，按胜场→胜率→场次排序） */
export async function fetchLeaderboard(): Promise<LeaderboardRowDto[]> {
  const data = await http.get<LeaderboardRowDto[]>('/leaderboard');
  if (!isArray(data) || !data.every(isObject)) throw new Error('排行榜数据格式异常');
  return data;
}

/**
 * GET /api/game/stream —— SSE 实时事件流（轮询的替代；单向推送，人类动作仍走 POST）。
 * 帧体是裸 GameStatusDto（不包 ApiResponse 信封；变化才推，全量快照 ⇒ 按 seq 去重即无缺口）。
 * 帧解析/取消/错误归一化由 shared 的 createSseStream 承担；这里只做领域释义：
 * `event: gone`（对局不可得）与 `status === 'ended'`（终局帧）视为业务性关流，调用方无需重连。
 * 返回停止函数。
 */
export function streamGameStatus(
  locator: GameLocator | undefined,
  handlers: {
    /** 每个变化帧（全量快照，已解包） */
    onFrame: (status: GameStatusDto) => void;
    /** 流结束（server 主动关闭或本地 abort）；error 时调用方可重连 */
    onEnd: (reason: StreamEnd, raw?: unknown) => void;
  },
): () => void {
  return createSseStream<GameStatusDto>(
    '/api/game/stream',
    {
      onMessage: (status) => {
        handlers.onFrame(normalizeStatus(status));
        if (status.status === 'ended') handlers.onEnd('ended');
      },
      onEvent: (event) => {
        if (event === 'gone') handlers.onEnd('gone');
      },
      onEnd: (reason, error) => {
        if (reason === 'complete') {
          // complete = 服务端非终局关流（异常断开）：按可重连的 error 透出
          handlers.onEnd('error', error);
          return;
        }
        handlers.onEnd(reason, error);
      },
    },
    {
      query: { room: locator?.room, game: locator?.game },
      headers: { accept: 'text/event-stream' },
    },
  );
}
