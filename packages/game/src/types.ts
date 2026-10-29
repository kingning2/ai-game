/**
 * 游戏域类型 —— 本包是框架无关、传输无关的对局基类：
 * 不含后端契约（DTO/端点归调用方），只定义「事件流 + 舞台投影」需要的最小结构。
 */
import type { JsonValue } from '@agent-packages/shared';
import { GameEventKind } from './enums';

/**
 * 对局事件（事件流的一行）。
 * kind 用开放联合：枚举覆盖已知取值，引擎新增事件时前端不炸、走 text 兜底展示。
 */
export type GameEventDto = {
  seq: number;
  day: number;
  kind: GameEventKind | (string & {});
  seat: number | null;
  text: string;
  /** dawn：昨夜死亡座位列表（平安夜为空） */
  deaths?: number[];
  /** vote：平票（无人放逐） */
  tie?: boolean;
  /** vote 放逐座位 / hunter 枪击目标座位 / duel 败者座位 */
  target?: number;
};

/** 舞台投影的最小玩家结构（调用方可传 richer 的领域 Player，泛型透传） */
export type GamePlayer = {
  id: number;
  name: string;
  status: 'alive' | 'dead' | (string & {});
};

/**
 * 喂给基类的数据（ingest 的入参）：调用方把拉到的数据原样喂进来即可，
 * 基类内部按 seq 去重增量累积，不关心数据来自 HTTP 轮询、WebSocket 还是测试桩。
 */
export type GameFeed<TStatus = JsonValue> = {
  /** 全量事件流（服务端可任意超前，基类只取新鲜部分） */
  events: GameEventDto[];
  /** 数据源的最新状态原样透传（基类不解读，UI 层自取 winner / mvp 等扩展字段） */
  status?: TStatus;
};

/** 事件前缀 → 舞台视图（projectStage 的返回；TPlayer 泛型保留调用方的领域字段） */
export type ProjectedStage<TPlayer extends GamePlayer = GamePlayer> = {
  phase: 'night' | 'day';
  phaseLabel: string;
  caption: string;
  stagePlayers: TPlayer[];
  speaker: TPlayer | undefined;
  speechText: string | undefined;
  spotlightSeats: number[];
  hostMessage: string;
};
