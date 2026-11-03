import type { BoardDef } from '../boards';

/** 角色中文名（介绍/对局舞台共用展示元数据） */
export const ROLE_LABEL: Record<string, string> = {
  werewolf: '狼人',
  seer: '预言家',
  witch: '女巫',
  guard: '守卫',
  hunter: '猎人',
  villager: '平民',
  idiot: '白痴',
  knight: '骑士',
};

/** 角色阵营归属：神职（蓝金）/ 狼人 / 平民侧（含白痴、骑士等特殊好人） */
const GOD_ROLES = new Set(['seer', 'witch', 'guard', 'hunter']);

export type CampTone = 'god' | 'wolf' | 'folk';
export type CampRow = { tone: CampTone; label: string; text: string };

/**
 * 阵营展示行（介绍页剧本面板）：神职 / 狼人 / 平民侧（含特殊好人牌）。
 * 特殊角色（白痴、骑士）跟随平民阵营展示，与引擎阵营判定一致。
 */
export function campRows(board: BoardDef): CampRow[] {
  const count = (role: string) => board.roles.filter((r) => r === role).length;
  const gods = board.roles.filter((r) => GOD_ROLES.has(r));
  const specials = board.roles.filter(
    (r) => !GOD_ROLES.has(r) && r !== 'werewolf' && r !== 'villager',
  );
  const rows: CampRow[] = [];
  if (gods.length > 0) {
    rows.push({ tone: 'god', label: '神职阵营', text: gods.map((r) => ROLE_LABEL[r]).join('+') });
  }
  rows.push({ tone: 'wolf', label: '狼人阵营', text: `狼人×${count('werewolf')}` });
  const folkText = [
    count('villager') > 0 ? `平民×${count('villager')}` : '',
    ...specials.map((r) => ROLE_LABEL[r]),
  ]
    .filter(Boolean)
    .join(' ');
  if (folkText) rows.push({ tone: 'folk', label: '平民阵营', text: folkText });
  return rows;
}

/** 规则列表：server 剧本为事实源；离线回退时给引擎夜行动顺序 + 胜负判定兜底 */
export function ruleLinesOf(board: BoardDef): string[] {
  if (board.rules?.length) return board.rules;
  const night = [
    '狼人刀人',
    '预言家验人',
    ...(board.guardSeat !== undefined ? ['守卫守护'] : []),
    ...(board.witchSeat !== undefined ? ['女巫用药'] : []),
  ].join(' → ');
  return [
    `夜晚：${night}`,
    '白天：依次发言，全员投票放逐一人',
    '胜负：狼全出局则好人胜，狼数≥好人则狼胜',
  ];
}
