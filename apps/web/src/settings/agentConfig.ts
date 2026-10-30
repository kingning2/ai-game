import type {
  AgentSettingsDraft,
  AgentSettingsValue,
} from '@agent-packages/shared-ui';
import { fetchAgentConfigDto, putAgentConfig } from '../apis/settings';
import type { AgentConfigDto, AgentConfigUpdate, SettingsPatchDto } from '../apis/types';

/**
 * 全局 agent 配置的读写字段 —— 传输已收口到 apis/settings，本文件只做
 * snake_case DTO → 设置面板受控值（camelCase）的归一映射。
 */

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
  return toValue(await fetchAgentConfigDto());
}

/** 保存草稿：Key 留空 = 不下发（服务端保留现值）；仅更新当前 provider 视角 */
export async function updateAgentConfig(draft: AgentSettingsDraft): Promise<AgentSettingsValue> {
  const providerView: SettingsPatchDto = { base_url: draft.baseUrl, model: draft.model };
  const apiKey = draft.apiKey.trim();
  if (apiKey !== '') providerView.api_key = apiKey;
  const body: AgentConfigUpdate = {
    provider: draft.provider,
    max_tokens: draft.maxTokens,
    [draft.provider]: providerView,
  };
  return toValue(await putAgentConfig(body));
}
