/** 狼人杀观战页 —— 领域类型（引擎接入前由 mock 数据驱动布局） */

/** 角色：狼人 / 预言家 / 女巫 / 守卫 / 猎人 / 平民 */
export type Role = 'werewolf' | 'seer' | 'witch' | 'guard' | 'hunter' | 'villager';

export type PlayerStatus = 'alive' | 'dead';

/** 主持人：流程声部，不参与博弈（引擎接入后由专职主持 Agent 扮演，负责发令与宣布结果） */
export type Host = {
  name: string;
  /** 头像回退（emoji / 首字母） */
  avatarFallback?: string;
};

export type Player = {
  id: number;
  /** 扮演的 Agent 名（布局阶段直接用模型名，接入引擎后换实际 Agent 标识） */
  name: string;
  /** 头像图（官方站点扒的 logo，位于 public/avatars/） */
  avatarSrc?: string;
  role: Role;
  status: PlayerStatus;
  /** 正在发言（上帝视角高亮当前发言者） */
  speaking?: boolean;
  /** 最近动作一行字 */
  note?: string;
};

/** 引擎某一刻的对局快照 —— 页面完全受控渲染，不持有内部状态 */
export type GameSnapshot = {
  players: Player[];
  /** 主持人 */
  host?: Host;
  /** 主持人当前主持词（天黑请闭眼 / 请发言 / 请投票等流程指令） */
  hostMessage?: string;
  /** 当前发言者（白天发言阶段才有） */
  currentSpeaker?: { playerId: number; text: string };
  /** 狼队夜聊耳语（夜晚阶段，上帝观战可见；接入引擎后由 WolfChatSpoke 事件驱动） */
  wolfChat?: { playerId: number; text: string };
  /** 对局阶段：夜晚叠夜幕遮罩 + 横幅；缺省不渲染阶段 UI */
  phase?: 'night' | 'day';
  /** 阶段横幅文案（如"天黑请闭眼""第 2 天 · 白天发言"）；缺省用内置文案 */
  phaseLabel?: string;
  /** 底部大字幕（旁白 / 天亮宣言） */
  caption?: string;
};
