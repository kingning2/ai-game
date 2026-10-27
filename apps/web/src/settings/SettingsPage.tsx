import {
  Button,
  SettingsPanel,
  Spinner,
  type AgentSettingsDraft,
  type AgentSettingsValue,
} from '@agent-packages/shared-ui';
import { useEffect, useState } from 'react';
import { fetchAgentConfig, updateAgentConfig } from './agentConfig';

export type SettingsPageProps = {
  onBack: () => void;
  /** 打开选手模型配置页（12 名参赛 AI 各自的 provider/模型/Key）；未传时不渲染入口 */
  onOpenRoster?: () => void;
  className?: string;
};

/**
 * AI 设置页（跨游戏全局）：模型供应商 / Base URL / 模型 / API Key / Max Tokens。
 * 配置存 server 端（PUT /api/agent/config），所有游戏的 Agent 决策共用；
 * Key 脱敏存储永不回显明文，留空保存 = 保留现有 Key。
 */
export function SettingsPage({ onBack, onOpenRoster, className }: SettingsPageProps) {
  const [value, setValue] = useState<AgentSettingsValue | null>(null);
  const [draft, setDraft] = useState<AgentSettingsDraft | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const config = await fetchAgentConfig();
        if (cancelled) return;
        const view = config.provider === 'openai' ? config.openai : config.anthropic;
        setValue(config);
        setDraft({
          provider: config.provider,
          baseUrl: view.baseUrl,
          model: view.model,
          apiKey: '',
          maxTokens: config.maxTokens,
        });
      } catch (err: unknown) {
        if (!cancelled) setError(err instanceof Error ? err.message : '配置加载失败');
      }
    }
    void load();
    return () => {
      cancelled = true;
    };
  }, []);

  async function handleSave(current: AgentSettingsDraft) {
    setSaving(true);
    setError(null);
    setSaved(false);
    try {
      const config = await updateAgentConfig(current);
      const view = config.provider === 'openai' ? config.openai : config.anthropic;
      setValue(config);
      setDraft({
        provider: config.provider,
        baseUrl: view.baseUrl,
        model: view.model,
        apiKey: '',
        maxTokens: config.maxTokens,
      });
      setSaved(true);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : '保存失败');
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className={`min-h-dvh bg-secondary-background ${className ?? ''}`}>
      <header className="sticky top-0 z-40 border-b border-separator bg-secondary-background">
        <div className="mx-auto flex h-14 w-full max-w-md items-center px-6">
          <Button variant="plain" size="sm" onClick={onBack}>
            ‹ 返回大厅
          </Button>
          <span className="flex-1 text-center text-sm font-semibold text-label">AI 设置</span>
          {onOpenRoster ? (
            <Button variant="plain" size="sm" onClick={onOpenRoster} className="w-20">
              选手模型
            </Button>
          ) : (
            <span className="w-20" aria-hidden />
          )}
        </div>
      </header>
      <div className="mx-auto flex max-w-md flex-col gap-3 px-6 py-8">
        <p className="text-xs leading-relaxed text-secondary-label">
          配置对全部游戏生效：Agent 决策、主持人播报、记忆压缩共用同一套模型配置。API Key
          保存在服务端，仅脱敏回显。
        </p>
        {error && !value ? (
          <p className="text-sm text-red" role="alert">
            {error}
          </p>
        ) : null}
        {!value || !draft ? (
          <div className="flex items-center justify-center gap-2 py-12 text-sm text-secondary-label">
            <Spinner size="sm" /> 加载中…
          </div>
        ) : (
          <>
            <SettingsPanel
              value={value}
              draft={draft}
              onDraftChange={(next) => {
                setSaved(false);
                setDraft(next);
              }}
              onSave={(next) => {
                void handleSave(next);
              }}
              saving={saving}
              error={error}
            />
            {saved && <output className="text-xs text-green">已保存，新配置即时生效</output>}
          </>
        )}
      </div>
    </div>
  );
}
