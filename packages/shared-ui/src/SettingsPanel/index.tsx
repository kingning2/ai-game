import type { HTMLAttributes, ReactNode } from 'react';
import { Button } from '../Button';
import { Card } from '../Card';
import { Input } from '../Input';
import { SegmentedControl, type SegmentedOption } from '../SegmentedControl';

/** 模型供应商（与桌面壳 Rust 端 /api/agent/config 的 provider 枚举对齐） */
export type AgentProviderKind = 'openai' | 'anthropic';

/** 单个供应商视角的生效配置 */
export type AgentProviderSettings = {
  baseUrl: string;
  model: string;
};

/** GET /api/agent/config 返回的生效配置（api_key 不回传前端） */
export type AgentSettingsValue = {
  provider: AgentProviderKind;
  openai: AgentProviderSettings;
  anthropic: AgentProviderSettings;
  maxTokens: number;
};

/** 表单草稿：apiKey 留空 = 保留现有 Key（下发语义由调用方决定） */
export type AgentSettingsDraft = {
  provider: AgentProviderKind;
  baseUrl: string;
  model: string;
  apiKey: string;
  maxTokens: number;
};

const PROVIDERS: ReadonlyArray<SegmentedOption<AgentProviderKind>> = [
  { value: 'openai', label: 'OpenAI' },
  { value: 'anthropic', label: 'Anthropic' },
];

export type SettingsPanelProps = HTMLAttributes<HTMLDivElement> & {
  /** 当前生效配置（切换供应商时用于带入对应视角的 baseUrl / model） */
  value: AgentSettingsValue;
  /** 受控表单草稿 */
  draft: AgentSettingsDraft;
  onDraftChange: (draft: AgentSettingsDraft) => void;
  /** 保存（Key 留空 = 保留现有 Key） */
  onSave: (draft: AgentSettingsDraft) => void;
  saving?: boolean;
  error?: string | null;
};

/** 表单字段行：小标签 + 控件（零内部状态，全部受控） */
function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-xs text-secondary-label">{label}</span>
      {children}
    </label>
  );
}

/**
 * Agent 设置面板 —— 模型供应商 / Base URL / 模型 / API Key / Max Tokens。
 * 组件零业务逻辑与零内部状态：请求、持久化、错误归一全部由调用方 hook 完成，
 * 本组件只做受控渲染与草稿字段映射。
 */
export function SettingsPanel({
  value,
  draft,
  onDraftChange,
  onSave,
  saving = false,
  error,
  className,
  ...props
}: SettingsPanelProps) {
  const switchProvider = (provider: AgentProviderKind) => {
    // 切换供应商时从生效配置带入对应视角，避免把 openai 的 URL 留在 anthropic 草稿里
    const view = provider === 'openai' ? value.openai : value.anthropic;
    onDraftChange({ ...draft, provider, baseUrl: view.baseUrl, model: view.model });
  };

  return (
    <Card className={className} {...props}>
      <div className="flex flex-col gap-3">
        <Field label="模型供应商">
          <SegmentedControl
            aria-label="模型供应商"
            options={PROVIDERS}
            value={draft.provider}
            onChange={switchProvider}
          />
        </Field>
        <Field label="Base URL">
          <Input
            value={draft.baseUrl}
            onChange={(e) => onDraftChange({ ...draft, baseUrl: e.target.value })}
            placeholder="https://api.openai.com/v1"
            spellCheck={false}
            autoComplete="off"
          />
        </Field>
        <Field label="模型">
          <Input
            value={draft.model}
            onChange={(e) => onDraftChange({ ...draft, model: e.target.value })}
            placeholder="gpt-4o"
            spellCheck={false}
            autoComplete="off"
          />
        </Field>
        <Field label="API Key">
          <Input
            type="password"
            value={draft.apiKey}
            onChange={(e) => onDraftChange({ ...draft, apiKey: e.target.value })}
            placeholder="留空保留现有 Key"
            autoComplete="new-password"
          />
        </Field>
        <Field label="Max Tokens">
          <Input
            type="number"
            min={1}
            value={draft.maxTokens}
            onChange={(e) => {
              const n = e.target.valueAsNumber;
              onDraftChange({ ...draft, maxTokens: Number.isNaN(n) ? 0 : n });
            }}
          />
        </Field>
        {error ? (
          <p className="text-xs text-red" role="alert">
            {error}
          </p>
        ) : null}
        <div className="flex items-center justify-end">
          <Button variant="solid" size="sm" disabled={saving} onClick={() => onSave(draft)}>
            {saving ? '保存中…' : '保存'}
          </Button>
        </div>
      </div>
    </Card>
  );
}
