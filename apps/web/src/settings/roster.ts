import type { AgentProviderKind } from '@agent-packages/shared-ui';

/** 统一响应包裹：code === 0 表示成功 */
type ApiEnvelope<T> = { code: number; message: string; data: T };

function isEnvelope<T>(value: unknown): value is ApiEnvelope<T> {
  return (
    typeof value === 'object' &&
    value !== null &&
    'code' in value &&
    'message' in value &&
    'data' in value
  );
}

async function unwrap<T>(response: Response): Promise<T> {
  const raw: unknown = await response.json();
  if (!isEnvelope<T>(raw)) throw new Error('响应不是合法的 ApiResponse 包裹');
  if (raw.code !== 0) throw new Error(raw.message);
  return raw.data;
}

/** 单供应商视角的有效参数（GET /api/roster 内嵌；Key 只出脱敏形态） */
export type RosterProviderSettings = {
  base_url: string;
  model: string;
  api_key_masked: string;
  has_key: boolean;
};

/** 覆盖合并进全局后的有效配置 */
export type RosterEffective = {
  provider: AgentProviderKind;
  openai: RosterProviderSettings;
  anthropic: RosterProviderSettings;
  max_tokens: number;
};

/** 单协议连接参数覆盖：字段缺省 = 继承全局 */
export type SettingsPatchDto = {
  base_url?: string;
  model?: string;
  api_key?: string;
};

/** 选手级覆盖：字段缺省 = 继承全局；全字段缺省 = 清除覆盖回全局 */
export type PlayerOverrideDto = {
  provider?: AgentProviderKind;
  openai?: SettingsPatchDto;
  anthropic?: SettingsPatchDto;
  max_tokens?: number;
  /** 自定义头像（emoji 或图片 URL）；缺省 = 无自定义 */
  avatar?: string;
};

/** GET /api/roster 元素 */
export type RosterPlayer = {
  name: string;
  /** 自定义头像；null = 用静态资源/首字兑底 */
  avatar: string | null;
  has_override: boolean;
  player_override: PlayerOverrideDto | null;
  effective: RosterEffective;
};

/** 拉取全员选手配置（12 人；有效配置已合并覆盖） */
export async function fetchRoster(): Promise<RosterPlayer[]> {
  const response = await fetch('/api/roster');
  return unwrap<RosterPlayer[]>(response);
}

/** 保存单个选手的覆盖（字段缺省 = 继承全局；全空对象 = 清除覆盖回全局）。名单外选手自动创建 */
export async function putPlayerOverride(
  name: string,
  override: PlayerOverrideDto,
): Promise<RosterPlayer> {
  const response = await fetch(`/api/roster/${encodeURIComponent(name)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(override),
  });
  return unwrap<RosterPlayer>(response);
}

/** 删除自建选手（内置选手 400），返回删除后的全员列表 */
export async function deletePlayer(name: string): Promise<RosterPlayer[]> {
  const response = await fetch(`/api/roster/${encodeURIComponent(name)}`, {
    method: 'DELETE',
  });
  return unwrap<RosterPlayer[]>(response);
}
