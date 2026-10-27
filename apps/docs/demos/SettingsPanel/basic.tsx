import { useState } from 'react';
import {
  SettingsPanel,
  type AgentSettingsDraft,
  type AgentSettingsValue,
} from '@agent-packages/shared-ui';

const INITIAL: AgentSettingsValue = {
  provider: 'openai',
  openai: {
    baseUrl: 'https://api.openai.com/v1',
    model: 'gpt-4o-mini',
    apiKeyMasked: '****cdef',
    hasKey: true,
  },
  anthropic: {
    baseUrl: 'https://api.anthropic.com',
    model: 'claude-sonnet-4-5',
    apiKeyMasked: '',
    hasKey: false,
  },
  maxTokens: 4096,
};

function draftFrom(
  value: AgentSettingsValue,
  provider: AgentSettingsValue['provider'],
): AgentSettingsDraft {
  const view = provider === 'openai' ? value.openai : value.anthropic;
  return {
    provider,
    baseUrl: view.baseUrl,
    model: view.model,
    apiKey: '',
    maxTokens: value.maxTokens,
  };
}

/** 基础用法：受控草稿 + 模拟保存（Key 只见脱敏值，留空 = 保持现有 Key） */
export default function SettingsPanelBasic() {
  const [value, setValue] = useState(INITIAL);
  const [draft, setDraft] = useState<AgentSettingsDraft>(() => draftFrom(INITIAL, 'openai'));
  const [saving, setSaving] = useState(false);

  const save = (next: AgentSettingsDraft) => {
    setSaving(true);
    // 模拟 PUT /api/agent/config：1s 后用新的脱敏态回填
    window.setTimeout(() => {
      const view = next.provider === 'openai' ? INITIAL.openai : INITIAL.anthropic;
      const masked = next.apiKey ? `****${next.apiKey.slice(-4)}` : view.apiKeyMasked;
      const hasKey = next.apiKey.length > 0 || view.hasKey;
      setValue((prev) => ({
        ...prev,
        provider: next.provider,
        maxTokens: next.maxTokens,
        [next.provider]: { baseUrl: next.baseUrl, model: next.model, apiKeyMasked: masked, hasKey },
      }));
      setDraft(draftFrom({ ...INITIAL, maxTokens: next.maxTokens }, next.provider));
      setSaving(false);
    }, 1000);
  };

  return (
    <SettingsPanel
      className="max-w-md"
      value={value}
      draft={draft}
      onDraftChange={setDraft}
      onSave={save}
      saving={saving}
    />
  );
}
