import type {
  AgentProviderKind,
  AgentSettingsDraft,
  AgentSettingsValue,
} from '@agent-packages/shared-ui';

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

/** 单供应商视角的 snake_case DTO（GET /api/agent/config） */
type ProviderSettingsDto = {
  base_url: string;
  model: string;
  api_key_masked: string;
  has_key: boolean;
};

type AgentConfigDto = {
  provider: AgentProviderKind;
  openai: ProviderSettingsDto;
  anthropic: ProviderSettingsDto;
  max_tokens: number;
};

type ProviderSettingsUpdate = {
  base_url?: string;
  api_key?: string;
  model?: string;
};

type AgentConfigUpdate = {
  provider?: AgentProviderKind;
  openai?: ProviderSettingsUpdate;
  anthropic?: ProviderSettingsUpdate;
  max_tokens?: number;
};

function toValue(dto: AgentConfigDto): AgentSettingsValue {
  return {
    provider: dto.provider,
    openai: { baseUrl: dto.openai.base_url, model: dto.openai.model },
    anthropic: { baseUrl: dto.anthropic.base_url, model: dto.anthropic.model },
    maxTokens: dto.max_tokens,
  };
}

/** 读取当前生效配置并归一为面板受控值 */
export async function fetchAgentConfig(): Promise<AgentSettingsValue> {
  const response = await fetch('/api/agent/config');
  return unwrap<AgentConfigDto>(response).then(toValue);
}

/** 保存草稿：Key 留空 = 不下发（服务端保留现值）；仅更新当前 provider 视角 */
export async function updateAgentConfig(draft: AgentSettingsDraft): Promise<AgentSettingsValue> {
  const providerView: ProviderSettingsUpdate = {
    base_url: draft.baseUrl,
    model: draft.model,
    ...(draft.apiKey.trim() === '' ? {} : { api_key: draft.apiKey.trim() }),
  };
  const body: AgentConfigUpdate = {
    provider: draft.provider,
    max_tokens: draft.maxTokens,
    [draft.provider]: providerView,
  };
  const response = await fetch('/api/agent/config', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  return unwrap<AgentConfigDto>(response).then(toValue);
}
