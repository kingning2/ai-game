import type { Player, Role } from './types';

/** 角色中文名（大厅角色 chips 共用） */
export const ROLE_CN: Record<Role, string> = {
  werewolf: '狼人',
  seer: '预言家',
  witch: '女巫',
  guard: '守卫',
  hunter: '猎人',
  villager: '平民',
};

/** 玩法板子：角色序列按座位 1..N 发放，夜间流程参数供剧本生成 */
export type BoardDef = {
  id: string;
  name: string;
  total: number;
  desc: string;
  roles: Role[];
  wolfSeats: number[];
  seerSeat: number;
  witchSeat?: number;
  guardSeat?: number;
  /** 女巫是否交解药（决定天亮是平安夜还是死讯） */
  witchSaves: boolean;
  killTarget: number;
  checkTarget: number;
};

export const BOARDS: BoardDef[] = [
  {
    id: 'rookie6',
    name: '新手 6 人局',
    total: 6,
    desc: '2 狼 · 预言家 · 女巫 · 2 平民',
    roles: ['werewolf', 'seer', 'villager', 'werewolf', 'villager', 'witch'],
    wolfSeats: [1, 4],
    seerSeat: 2,
    witchSeat: 6,
    witchSaves: true,
    killTarget: 3,
    checkTarget: 5,
  },
  {
    id: 'standard9',
    name: '标准 9 人局',
    total: 9,
    desc: '3 狼 · 预言家 · 女巫 · 猎人 · 3 平民',
    roles: [
      'werewolf',
      'seer',
      'villager',
      'werewolf',
      'hunter',
      'villager',
      'werewolf',
      'witch',
      'villager',
    ],
    wolfSeats: [1, 4, 7],
    seerSeat: 2,
    witchSeat: 8,
    witchSaves: true,
    killTarget: 6,
    checkTarget: 9,
  },
  {
    id: 'allstars12',
    name: '全明星 12 人局',
    total: 12,
    desc: '4 狼 · 预言家 · 女巫 · 守卫 · 猎人 · 4 平民',
    roles: [
      'werewolf',
      'witch',
      'villager',
      'seer',
      'werewolf',
      'villager',
      'villager',
      'guard',
      'villager',
      'werewolf',
      'werewolf',
      'hunter',
    ],
    wolfSeats: [1, 5, 10, 11],
    seerSeat: 4,
    witchSeat: 2,
    guardSeat: 8,
    witchSaves: false,
    killTarget: 3,
    checkTarget: 7,
  },
];

/** 选手池：12 名 AI（名字 + 头像 logo，位于 public/avatars/）；房间向导选人也从这里取 */
export const AGENT_POOL: { name: string; avatarSrc: string }[] = [
  { name: 'Claude Opus 5.5', avatarSrc: '/avatars/claude.png' },
  { name: 'Kimi K3.1 Pro', avatarSrc: '/avatars/kimi.png' },
  { name: 'Gemini 4 Argon', avatarSrc: '/avatars/gemini.png' },
  { name: 'GPT-6.1', avatarSrc: '/avatars/openai.png' },
  { name: 'Grok 4.7', avatarSrc: '/avatars/grok.png' },
  { name: 'MiMo V2.6', avatarSrc: '/avatars/mimo.png' },
  { name: 'Qwen3.8 Max', avatarSrc: '/avatars/qwen.svg' },
  { name: 'DeepSeek V4.1', avatarSrc: '/avatars/deepseek.ico' },
  { name: 'GLM-5.3', avatarSrc: '/avatars/chatglm.png' },
  { name: 'MiniMax M3.1', avatarSrc: '/avatars/minimax.png' },
  { name: '豆包 Seed 2.1', avatarSrc: '/avatars/doubao.png' },
  { name: 'Mistral Large 4', avatarSrc: '/avatars/mistral.png' },
];

/** 按板子生成对局玩家：角色按座位 1..N 发放，全员存活（剧本内的死亡由步骤下发） */
export function buildPlayers(board: BoardDef): Player[] {
  return AGENT_POOL.slice(0, board.total).map((p, i) => ({
    id: i + 1,
    name: p.name,
    avatarSrc: p.avatarSrc,
    role: board.roles[i],
    status: 'alive' as const,
  }));
}

/** 发牌结果：洗牌后的板子（座位参数同步重派生）+ 入座玩家 */
export type DealtGame = {
  board: BoardDef;
  players: Player[];
  /** 真实房间号（SQLite 落库后回填；undefined = 房间服务不可用降级） */
  roomCode?: string;
};

// 真实房间号（SQLite 落库后回填；undefined = 房间服务不可用降级）

/** 从座位池随机取一个（剧本刀/验目标） */
function pickSeat(pool: number[]): number {
  const idx = Math.floor(Math.random() * pool.length);
  return pool[idx] ?? pool[0];
}

/** 角色显示名 → 角色代码（server seat_roles 反查） */
const ROLE_BY_LABEL: Record<string, Role> = {
  狼人: 'werewolf',
  预言家: 'seer',
  女巫: 'witch',
  守卫: 'guard',
  猎人: 'hunter',
  平民: 'villager',
};

/**
 * 采用 server 真实发牌：座位名单与身份全部来自引擎（预热开局后拉取），
 * 保证观战界面的身份牌与真实对局一致。
 */
export function dealtFromServer(
  board: BoardDef,
  seats: string[],
  seatRoles: string[],
  roomCode?: string,
): DealtGame {
  const roles = seatRoles.map((label) => ROLE_BY_LABEL[label] ?? 'villager');
  const seatOf = (role: Role): number[] =>
    roles.reduce<number[]>((acc, r, i) => (r === role ? [...acc, i + 1] : acc), []);
  const goodSeats = roles.reduce<number[]>(
    (acc, r, i) => (r !== 'werewolf' ? [...acc, i + 1] : acc),
    [],
  );
  const withRoles: BoardDef = {
    ...board,
    roles,
    wolfSeats: seatOf('werewolf'),
    seerSeat: seatOf('seer')[0],
    witchSeat: seatOf('witch')[0],
    guardSeat: seatOf('guard')[0],
    killTarget: pickSeat(goodSeats),
    checkTarget: pickSeat(goodSeats),
  };
  const players: Player[] = seats.map((name, i) => ({
    id: i + 1,
    name,
    avatarSrc: AGENT_POOL.find((a) => a.name === name)?.avatarSrc ?? '',
    role: roles[i] ?? 'villager',
    status: 'alive' as const,
  }));
  return { board: withRoles, players, roomCode };
}

/** Fisher-Yates 原地洗牌（副本上操作） */
function shuffle<T>(items: T[]): T[] {
  const out = [...items];
  for (let i = out.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1));
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}

/**
 * 随机发牌（server 不可达时的本地回退）：洗 role 序列 + 按新角色分布重派生剧本座位参数，
 * 刀/验目标重随机到非狼座位（剧本语义：狼刀目标、预言家验出好人）。
 * members 为选中的选手（按入座顺序），数量须等于 board.total。
 */
export function dealGame(
  board: BoardDef,
  members: { name: string; avatarSrc: string }[],
): DealtGame {
  const roles = shuffle(board.roles);
  const seatOf = (role: Role): number[] =>
    roles.reduce<number[]>((acc, r, i) => (r === role ? [...acc, i + 1] : acc), []);
  const goodSeats = roles.reduce<number[]>(
    (acc, r, i) => (r !== 'werewolf' ? [...acc, i + 1] : acc),
    [],
  );
  return {
    board: {
      ...board,
      roles,
      wolfSeats: seatOf('werewolf'),
      seerSeat: seatOf('seer')[0],
      witchSeat: seatOf('witch')[0],
      guardSeat: seatOf('guard')[0],
      killTarget: pickSeat(goodSeats),
      checkTarget: pickSeat(goodSeats),
    },
    players: members.map((m, i) => ({
      id: i + 1,
      name: m.name,
      avatarSrc: m.avatarSrc,
      role: roles[i],
      status: 'alive' as const,
    })),
  };
}
