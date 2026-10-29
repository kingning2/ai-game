import { useState, type HTMLAttributes, type InputHTMLAttributes, type ReactNode } from 'react';
import { Button } from '../Button';
import { Card } from '../Card';
import { Divider } from '../layout/Divider';
import { Spinner } from '../Spinner';
import { Stack } from '../layout/Stack';
import { SegmentedControl, type SegmentedOption } from '../SegmentedControl';
import { Slider } from '../Slider';
import { cn } from '../cn';

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

const PROVIDER_HINTS: Record<AgentProviderKind, string> = {
  openai: 'OpenAI 官方及任何 OpenAI 兼容端点均可',
  anthropic: 'Anthropic Messages API 及兼容网关均可',
};

/** Max Tokens 滑杆范围（128 的倍数，覆盖常见单轮输出上限） */
const MAX_TOKENS_RANGE = { min: 256, max: 16384, step: 128 } as const;

/** 内联线稿图标（lucide 风格，size 16 / strokeWidth 1.8） */
function EyeIcon({ off = false }: { off?: boolean }) {
  return (
    <svg
      aria-hidden
      viewBox="0 0 24 24"
      width={16}
      height={16}
      fill="none"
      stroke="currentColor"
      strokeWidth={1.8}
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {off ? (
        <>
          <path d="M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49" />
          <path d="M14.084 14.158a3 3 0 0 1-4.242-4.242" />
          <path d="M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143" />
          <path d="m2 2 20 20" />
        </>
      ) : (
        <>
          <path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0" />
          <circle cx="12" cy="12" r="3" />
        </>
      )}
    </svg>
  );
}

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

/** 分组：小标题 + 一张零内边距卡片（行自带内边距，分隔线走行内 inset） */
function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="flex flex-col gap-1.5">
      <h3 className="px-1 text-xs font-medium text-secondary-label">{title}</h3>
      <Card className="overflow-hidden p-0">{children}</Card>
    </section>
  );
}

/** 设置行：标签居左、控件居右、11px 最小行高（Apple 列表行规格） */
function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex min-h-11 items-center gap-3 px-4">
      <span className="shrink-0 text-sm text-label">{label}</span>
      <div className="flex min-w-0 flex-1 items-center justify-end gap-2">{children}</div>
    </div>
  );
}

/** 行内无框输入：右对齐融入列表行；无框但键盘焦点仍可见（可达性底线） */
function RowInput({ className, ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...props}
      className={cn(
        'h-9 w-full min-w-0 rounded-md bg-transparent text-right text-sm text-label',
        'placeholder:text-placeholder-text',
        'focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-tint',
        className,
      )}
    />
  );
}

/**
 * Agent 设置面板 —— 模型供应商 / Base URL / 模型 / API Key / Max Tokens。
 * 视觉为 iOS 设置页分组列表：每个配置域一组，行内标签居左、值居右。
 * 组件零业务逻辑与零内部数据状态：请求、持久化、错误归一全部由调用方 hook 完成，
 * 本组件只做受控渲染与草稿字段映射（Key 可见性是纯展示态）。
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
  const [keyVisible, setKeyVisible] = useState(false);

  const switchProvider = (provider: AgentProviderKind) => {
    // 切换供应商时从生效配置带入对应视角，避免把 openai 的 URL 留在 anthropic 草稿里
    const view = provider === 'openai' ? value.openai : value.anthropic;
    onDraftChange({ ...draft, provider, baseUrl: view.baseUrl, model: view.model });
  };

  return (
    <Stack gap={5} className={className} {...props}>
      <Section title="模型供应商">
        <div className="flex flex-col gap-2 px-4 py-3">
          <SegmentedControl
            aria-label="模型供应商"
            options={PROVIDERS}
            value={draft.provider}
            onChange={switchProvider}
            className="w-full"
          />
          <p className="text-xs text-secondary-label">{PROVIDER_HINTS[draft.provider]}</p>
        </div>
      </Section>

      <Section title="连接">
        <Row label="Base URL">
          <RowInput
            value={draft.baseUrl}
            onChange={(e) => onDraftChange({ ...draft, baseUrl: e.target.value })}
            placeholder="https://api.openai.com/v1"
            spellCheck={false}
            autoComplete="off"
          />
        </Row>
        <Divider inset />
        <Row label="模型">
          <RowInput
            value={draft.model}
            onChange={(e) => onDraftChange({ ...draft, model: e.target.value })}
            placeholder="gpt-4o"
            spellCheck={false}
            autoComplete="off"
          />
        </Row>
      </Section>

      <Section title="凭据">
        <Row label="API Key">
          <RowInput
            type={keyVisible ? 'text' : 'password'}
            value={draft.apiKey}
            onChange={(e) => onDraftChange({ ...draft, apiKey: e.target.value })}
            placeholder="留空保留现有 Key"
            autoComplete="new-password"
            className="text-right placeholder:text-right"
          />
          <button
            type="button"
            aria-label={keyVisible ? '隐藏 API Key' : '显示 API Key'}
            aria-pressed={keyVisible}
            onClick={() => setKeyVisible((visible) => !visible)}
            className="flex size-7 shrink-0 cursor-pointer items-center justify-center rounded-full text-tertiary-label transition-colors duration-200 ease-out hover:bg-fill hover:text-label focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-tint"
          >
            {keyVisible ? <EyeIcon off /> : <EyeIcon />}
          </button>
        </Row>
      </Section>

      <Section title="生成参数">
        <Row label="Max Tokens">
          <span className="text-sm tabular-nums text-secondary-label">{draft.maxTokens}</span>
        </Row>
        <div className="px-4 pb-4">
          <Slider
            aria-label="Max Tokens"
            min={MAX_TOKENS_RANGE.min}
            max={MAX_TOKENS_RANGE.max}
            step={MAX_TOKENS_RANGE.step}
            value={draft.maxTokens}
            onChange={(maxTokens) => onDraftChange({ ...draft, maxTokens })}
          />
          <div className="mt-1 flex justify-between text-[10px] tabular-nums text-tertiary-label">
            <span>{MAX_TOKENS_RANGE.min}</span>
            <span>{MAX_TOKENS_RANGE.max}</span>
          </div>
        </div>
      </Section>

      {error ? (
        <p className="px-1 text-xs text-red" role="alert">
          {error}
        </p>
      ) : null}

      <Button variant="solid" disabled={saving} onClick={() => onSave(draft)} className="w-full">
        {saving ? (
          <>
            <Spinner size="sm" className="text-on-accent" />
            保存中…
          </>
        ) : (
          '保存'
        )}
      </Button>
    </Stack>
  );
}
