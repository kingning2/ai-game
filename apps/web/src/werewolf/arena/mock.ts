import type { GameSnapshot, Host, Player } from './types';

/** 主持人（法官）：只主持流程，不下场博弈 */
export const MOCK_HOST: Host = {
  name: '主持人',
  avatarFallback: '🎩',
};

/** 舞台演示数据：12 名 Agent（4 狼 + 预/女/守/猎 4 神 + 4 民），第 2 天白天；头像为各官方站点 logo */
export const MOCK_PLAYERS: Player[] = [
  {
    id: 1,
    name: '豆包',
    avatarSrc: '/avatars/doubao.png',
    role: 'werewolf',
    status: 'alive',
    note: '藏得很深，全程好人脸',
  },
  {
    id: 2,
    name: '元宝',
    avatarSrc: '/avatars/yuanbao.png',
    role: 'witch',
    status: 'alive',
    note: '解药已用于第 1 夜',
  },
  {
    id: 3,
    name: '智谱清言',
    avatarSrc: '/avatars/chatglm.png',
    role: 'villager',
    status: 'alive',
    speaking: true,
    note: '正在发言',
  },
  {
    id: 4,
    name: 'DeepSeek V3',
    avatarSrc: '/avatars/deepseek.ico',
    role: 'seer',
    status: 'alive',
    note: '已验两人',
  },
  {
    id: 5,
    name: '通义千问',
    avatarSrc: '/avatars/qwen.svg',
    role: 'werewolf',
    status: 'dead',
    note: '被猎人开枪带走',
  },
  {
    id: 6,
    name: 'Gemini 2.5',
    avatarSrc: '/avatars/gemini.png',
    role: 'villager',
    status: 'dead',
    note: '第 1 夜被狼人击杀',
  },
  {
    id: 7,
    name: 'Kimi',
    avatarSrc: '/avatars/kimi.png',
    role: 'villager',
    status: 'alive',
    note: '已被预言家验为好人',
  },
  {
    id: 8,
    name: 'GPT-4o',
    avatarSrc: '/avatars/openai.png',
    role: 'guard',
    status: 'alive',
    note: '今晚计划守护预言家',
  },
  {
    id: 9,
    name: '文心一言',
    avatarSrc: '/avatars/ernie.png',
    role: 'villager',
    status: 'alive',
    note: '发言逻辑清晰',
  },
  {
    id: 10,
    name: 'DeepSeek R1',
    avatarSrc: '/avatars/deepseek.ico',
    role: 'werewolf',
    status: 'alive',
    note: '伪装村民，低调跟票',
  },
  {
    id: 11,
    name: 'GPT-5',
    avatarSrc: '/avatars/openai.png',
    role: 'werewolf',
    status: 'alive',
    note: '昨天带偏了投票节奏',
  },
  {
    id: 12,
    name: 'Grok 4',
    avatarSrc: '/avatars/grok.png',
    role: 'hunter',
    status: 'alive',
    note: '枪已出手',
  },
];

export const MOCK_DAY_GAME: GameSnapshot = {
  players: MOCK_PLAYERS,
  host: MOCK_HOST,
  hostMessage: '现在进入白天发言，从 3 号智谱清言开始，请发言的 Agent 注意时间。',
  currentSpeaker: {
    playerId: 3,
    text: '来来来，我们复盘一遍——1 号开场站边没问题，但人家跳预之后被回头刀，这局你们输在信息差。6 号被救回来那轮突然跳守卫，9 号才是真平民。12 号从头到尾都在装好人——但狼只剩一个：5 号通义千问，从头到尾都在给狼队当挡箭牌，昨天投票还把节奏带偏了。今天这票不用犹豫了。',
  },
  phase: 'day',
  phaseLabel: '第 2 天 · 白天发言',
  caption: '但最后好多一枪带走了个狼',
};

/** 夜晚演示快照：狼队夜聊（耳语仅狼队与上帝可见，观战页以节目视角直出） */
export const MOCK_NIGHT_GAME: GameSnapshot = {
  players: MOCK_PLAYERS.map((p) => ({ ...p, speaking: p.id === 1 })),
  host: MOCK_HOST,
  hostMessage: '天黑请闭眼。狼人请睁眼——请狼队商量今晚的猎物。',
  wolfChat: {
    playerId: 1,
    text: '今晚收掉 4 号。DeepSeek 连续两天跳预言家，报的金水全对得上，拖到后面我们必输；现在刀口最稳。',
  },
  phase: 'night',
  phaseLabel: '第 2 夜 · 狼队商议',
  caption: '狼人正在挑选今晚的猎物',
};

/** 页面当前演示的快照（切回白天演示改成 MOCK_DAY_GAME） */
export const MOCK_GAME: GameSnapshot = MOCK_NIGHT_GAME;
