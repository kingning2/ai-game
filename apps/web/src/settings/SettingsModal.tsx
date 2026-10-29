import {
  Button,
  SegmentedControl,
  SettingsPanel,
  Spinner,
  type AgentSettingsDraft,
  type AgentSettingsValue,
} from '@agent-packages/shared-ui';
import { useEffect, useState } from 'react';
import { cn } from '@agent-packages/shared-ui';
import { fetchAgentConfig, updateAgentConfig } from './agentConfig';
import { RosterManager } from './RosterManager';

export type SettingsModalProps = {
  open: boolean;
  onClose: () => void;
  className?: string;
};

type ModalSection = 'agent' | 'roster';

/** 选手模型分节的介绍已内置于 RosterManager，避免重复文案 */
const SECTION_HINTS: Partial<Record<ModalSection, string>> = {
  agent:
    '全局模型配置：Agent 决策、主持人播报、记忆压缩共用。Key 服务端脱敏存储，留空保存 = 保留现有 Key。',
};

/**
 * 设置弹层（主页/大厅 ⚙️ 入口）：AI 设置 + 选手模型两个分节，左侧导航右侧内容。
 * 遮罩点击 / Esc / 「完成」均可关闭；选手模型分节内嵌 RosterManager（与独立页同一内容组件）。
 */
export function SettingsModal({ open, onClose, className }: SettingsModalProps) {
  const [section, setSection] = useState<ModalSection>('agent');

  // ---- AI 设置分节状态（与 SettingsPage 同一套 SettingsPanel 契约） ----
  const [value, setValue] = useState<AgentSettingsValue | null>(null);
  const [draft, setDraft] = useState<AgentSettingsDraft | null>(null);
  const [agentError, setAgentError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (!open) return undefined;
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
        if (!cancelled) setAgentError(err instanceof Error ? err.message : '配置加载失败');
      }
    }
    void load();
    return () => {
      cancelled = true;
    };
  }, [open]);

  // Esc 关闭（仅弹层打开时）
  useEffect(() => {
    if (!open) return undefined;
    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape') onClose();
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, onClose]);

  if (!open) return null;

  async function handleSave(current: AgentSettingsDraft) {
    setSaving(true);
    setAgentError(null);
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
      setAgentError(err instanceof Error ? err.message : '保存失败');
    } finally {
      setSaving(false);
    }
  }

  const hint = SECTION_HINTS[section];

  return (
    <div
      role="presentation"
      className={cn('fixed inset-0 z-50 flex items-center justify-center p-6', className)}
      onClick={onClose}
    >
      {/* 遮罩 */}
      <div aria-hidden className="absolute inset-0 bg-black/40 backdrop-blur-[2px]" />
      {/* 弹层卡片 */}
      <div
        role="dialog"
        aria-modal="true"
        aria-label="设置"
        onClick={(event) => event.stopPropagation()}
        className="relative flex h-[80dvh] w-full max-w-2xl flex-col overflow-hidden rounded-2xl border border-separator bg-background shadow-2xl"
      >
        {/* 头部：标题 + 关闭 */}
        <header className="flex h-13 shrink-0 items-center justify-between border-b border-separator px-5">
          <span className="text-sm font-semibold text-label">设置</span>
          <Button variant="plain" size="sm" aria-label="关闭设置" onClick={onClose}>
            ✕
          </Button>
        </header>

        {/* 分节切换 */}
        <div className="shrink-0 px-5 pt-4">
          <SegmentedControl
            aria-label="设置分节"
            value={section}
            onChange={setSection}
            options={[
              { value: 'agent', label: 'AI 设置' },
              { value: 'roster', label: '选手模型' },
            ]}
          />
        </div>

        {/* 内容区：弹层内唯一滚动方 */}
        <div className="min-h-0 flex-1 overflow-y-auto px-5 py-4">
          {hint ? (
            <p className="mb-4 text-xs leading-relaxed text-secondary-label">{hint}</p>
          ) : null}
          {section === 'agent' ? (
            agentError && !value ? (
              <p className="text-sm text-destructive" role="alert">
                {agentError}
              </p>
            ) : !value || !draft ? (
              <div className="flex items-center justify-center gap-2 py-10 text-sm text-secondary-label">
                <Spinner size="sm" /> 加载中…
              </div>
            ) : (
              <div className="flex flex-col gap-3">
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
                  error={agentError}
                />
                {saved && <output className="text-xs text-green">已保存，新配置即时生效</output>}
              </div>
            )
          ) : (
            <RosterManager />
          )}
        </div>

        {/* 底部：完成 */}
        <footer className="flex shrink-0 items-center justify-end border-t border-separator px-5 py-3">
          <Button size="sm" onClick={onClose}>
            完成
          </Button>
        </footer>
      </div>
    </div>
  );
}
