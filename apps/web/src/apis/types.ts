/**
 * web ↔ rust/server 契约类型。
 *
 * 单一事实源：rust/server 的 utoipa 文档（`pnpm codegen` 一键导出）。
 * - 基础形状一律取自 ./schema.gen.ts（生成物，禁止手改），改契约只需改 Rust DTO 后重跑 codegen
 * - 本文件只剩两层手写内容：
 *   1. 语义收窄 —— server 出 string 的字段按前端消费收成字面量联合（`(string & {})` 保前向兼容）
 *   2. 跨包引用 —— GameEventDto 单一事实源在 @agent-packages/game；AgentProviderKind 在 shared-ui
 *   （ScriptId/ScriptDto/StreamEnd 无 server 端点，是纯前端契约，保留手写）
 */
import type { GameEventDto } from '@agent-packages/game';
import type { AgentProviderKind } from '@agent-packages/shared-ui';
import type { Role } from '../werewolf/types';
import type { components } from './schema.gen';

export type { GameEventDto };

type Schemas = components['schemas'];

/** server 剧本 id = 引擎板子预设名（GameState::resolve_board）；纯前端契约（boards.ts） */
export type ScriptId = 'newbie_six' | 'nine_standard' | 'all_stars' | (string & {});

/** server 剧本：一个剧本 = 一种玩法（板子 + 规则 + 主持开场词）；纯前端契约，server 无此端点 */
export type ScriptDto = {
  /** 剧本 id（开局传给 POST /api/game/start 的 board 字段） */
  id: ScriptId;
  name: string;
  /** 玩家座位数 */
  total: number;
  desc: string;
  /** 角色配比（Role 联合当前定义在 werewolf/types，仅类型引用无运行时依赖） */
  roles: { role: Role; name: string; count: number }[];
  /** 规则列表（介绍页逐条展示） */
  rules: string[];
  /** 主持开场词（介绍页逐段播报） */
  intro: string[];
};

/** 对局状态（GET /api/game/status，各 POST 端点同样返回）；形状 = Rust GameStatusDto */
export type GameStatusDto = Omit<
  Schemas['GameStatusDto'],
  'status' | 'winner' | 'ended_reason' | 'mvp' | 'events' | 'roles'
> & {
  /** 生命周期：idle / running / paused / ended */
  status: 'idle' | 'running' | 'paused' | 'ended' | (string & {});
  /** 胜方：good / werewolf；仅引擎自然结算后有值 */
  winner: 'good' | 'werewolf' | null;
  /** 结束原因：settled（自然分出胜负）/ manual（手动结束） */
  ended_reason: 'settled' | 'manual' | null;
  /** 赛后 MVP 投票：null = 无 MVP 流程（手动终局 / stale 中断局） */
  mvp?: Schemas['MvpDto'] | null;
  /** 上帝视角事件流：跨端共享契约，单一事实源在 @agent-packages/game */
  events: GameEventDto[];
  /** 各身份存活统计（Rust RoleStatusDto：role/name/total/alive） */
  roles: Schemas['RoleStatusDto'][];
  /**
   * 座位 → Edge 音色 id（每局随机分配，与 seats 对齐）。
   * ponytail: server GameStatusDto 尚未下发该字段（TTS 接入中），先按契约保留；接入后删此覆盖
   */
  seat_voices?: string[];
};

/** 单张 MVP 选票 */
export type MvpVoteDto = Schemas['MvpVoteDto'];

/** 赛后 MVP 投票状态：pending 投票中（票从 server 轮询）；done 已决出 */
export type MvpDto = Omit<Schemas['MvpDto'], 'phase'> & {
  phase: 'pending' | 'done' | (string & {});
};

/** 房间信息（SQLite 落库） */
export type RoomInfo = Schemas['RoomDto'];

/** 排行榜行：跨对局聚合（全部 finished 局 × 房间选手） */
export type LeaderboardRowDto = Schemas['LeaderboardEntryDto'];

/** 对局定位参数：?room=房间码 优先，或 ?game=对局 id；都缺省用 server 最近一局 */
export type GameLocator = Schemas['GameQuery'];

/** SSE 流断开原因：ended/gone 由 server 主动关闭（无需重连），error/complete 可重试；纯前端契约 */
export type StreamEnd = 'ended' | 'gone' | 'aborted' | 'error';

// ── 设置页（agent 配置 / 选手 roster）契约，端点见 ./settings.ts ──

/** 单供应商视角的 snake_case DTO（GET /api/agent/config 内嵌；Key 只出脱敏形态） */
export type ProviderSettingsDto = Schemas['ProviderSettingsDto'];

export type AgentConfigDto = Omit<Schemas['AgentConfigDto'], 'provider'> & {
  provider: AgentProviderKind;
};

/** 单协议连接参数覆盖：字段缺省 = 继承全局 */
export type SettingsPatchDto = Schemas['ProviderSettingsUpdate'];

export type AgentConfigUpdate = Omit<Schemas['AgentConfigUpdate'], 'provider'> & {
  provider?: AgentProviderKind;
};

/** 选手级覆盖：字段缺省 = 继承全局；全字段缺省 = 清除覆盖回全局 */
export type PlayerOverrideDto = AgentConfigUpdate & {
  /** 自定义头像（emoji 或图片 URL）；缺省 = 无自定义 */
  avatar?: string;
};

/** GET /api/roster 元素 */
export type RosterPlayer = Omit<Schemas['RosterPlayerDto'], 'avatar' | 'player_override'> & {
  /** 自定义头像；null = 用静态资源/首字兑底 */
  avatar: string | null;
  player_override: PlayerOverrideDto | null;
  /** 覆盖合并进全局后的有效配置（与全局配置同构） */
  effective: AgentConfigDto;
};
