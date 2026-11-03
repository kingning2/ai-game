/** 狼人杀观战页 —— 领域类型 */

/** 角色：狼人 / 预言家 / 女巫 / 守卫 / 猎人 / 平民 */
export type Role = 'werewolf' | 'seer' | 'witch' | 'guard' | 'hunter' | 'villager';

export type PlayerStatus = 'alive' | 'dead';

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
