/**
 * 狼人杀对局枚举：与 rust/games/werewolf/src/event.rs、engine.rs 的服务端契约对齐。
 * 枚举成员值即 server DTO 里的字符串字面量；未知取值按 string 兜底处理（见 types.ts 的开放联合）。
 */

/** 对局事件类型（GameEventDto.kind） */
export enum GameEventKind {
  /** 开局入座 */
  Started = 'started',
  /** 天黑请闭眼 */
  Night = 'night',
  /** 夜行动作（狼刀 / 守卫守 / 女巫用药 / 预言家查验，见 NightStage） */
  NightAction = 'night_action',
  /** 狼队夜聊耳语 */
  WolfChat = 'wolf_chat',
  /** 狼队统一意见、结束夜聊 */
  WolfPass = 'wolf_pass',
  /** 天亮请睁眼（带昨夜死讯） */
  Dawn = 'dawn',
  /** 白天发言 */
  Speech = 'speech',
  /** 平票决斗 */
  Duel = 'duel',
  /** 放逐投票 */
  Vote = 'vote',
  /** 白痴翻牌 */
  Idiot = 'idiot',
  /** 猎人开枪 */
  Hunter = 'hunter',
  /** 座位出局 */
  Died = 'died',
  /** 对局结束（胜负分出） */
  Over = 'over',
}

/** 夜晚行动窗口（engine.rs NightStage 的先后顺序） */
export enum NightStage {
  Guard = 'guard',
  WerewolfChat = 'werewolf_chat',
  Werewolves = 'werewolves',
  Witch = 'witch',
  Seer = 'seer',
}

/** 对局运行状态（GameStatusDto.status） */
export enum GameRunStatus {
  Idle = 'idle',
  Running = 'running',
  Paused = 'paused',
  Ended = 'ended',
}

/** 阵营 / 胜负 */
export enum Camp {
  Good = 'good',
  Werewolf = 'werewolf',
}

/** 玩家角色（引擎发牌真实身份；seat_roles 为中文显示名） */
export enum PlayerRole {
  Werewolf = 'werewolf',
  Seer = 'seer',
  Witch = 'witch',
  Guard = 'guard',
  Hunter = 'hunter',
  Idiot = 'idiot',
  Villager = 'villager',
}

/** 终局原因（GameStatusDto.ended_reason） */
export enum GameEndReason {
  /** 正常结算（胜负分出） */
  Settled = 'settled',
  /** 手动终局 */
  Manual = 'manual',
}

/** 房间状态（RoomInfo.status） */
export enum RoomStatus {
  Waiting = 'waiting',
  Playing = 'playing',
  Finished = 'finished',
}

/** 赛后 MVP 投票阶段（MvpDto.phase） */
export enum MvpPhase {
  /** 投票中（票从 server 轮询） */
  Pending = 'pending',
  /** 已决出 */
  Done = 'done',
}
