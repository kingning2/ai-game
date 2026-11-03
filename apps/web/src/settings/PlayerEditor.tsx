import { Avatar, Button, Card, Input, SegmentedControl, cn } from '@agent-packages/shared-ui';
import { AVATAR_CHOICES, isDraftDirty, type PlayerDraft } from './rosterDraft';
import type { RosterPlayer } from '../apis/types';

export type PlayerEditorProps = {
  /** true = 新增选手模式（可填名字）；false = 覆盖已有选手 */
  creating: boolean;
  /** 被编辑的选手名（非新增模式展示） */
  name: string;
  newName: string;
  onNewNameChange: (value: string) => void;
  draft: PlayerDraft;
  onDraftChange: (draft: PlayerDraft) => void;
  /** 默认头像回落展示（父组件算好：同名选手 avatar → AGENT_POOL 资源） */
  defaultAvatarSrc: string | undefined;
  /** 全局基线（placeholder 提示未覆盖时继承的值） */
  baseline: RosterPlayer['effective'] | undefined;
  saving: boolean;
  onSave: () => void;
  onRemove: () => void;
};

/** 选手覆盖配置编辑卡（纯展示，状态由 RosterManager 的 reducer 持有） */
export function PlayerEditor({
  creating,
  name,
  newName,
  onNewNameChange,
  draft,
  onDraftChange,
  defaultAvatarSrc,
  baseline,
  saving,
  onSave,
  onRemove,
}: PlayerEditorProps) {
  return (
    <Card className="w-full max-w-3xl">
      <div aria-label={creating ? '新增选手' : '选手覆盖配置'} className="flex flex-col gap-3.5">
        {creating ? (
          <label className="flex flex-col gap-1.5" htmlFor="new-player-name">
            <span className="text-xs font-medium text-secondary-label">选手名</span>
            <Input
              id="new-player-name"
              placeholder="例如：DeepSeek R2"
              value={newName}
              onChange={(event) => onNewNameChange(event.target.value)}
            />
          </label>
        ) : (
          <span className="text-base font-semibold text-label">{name}</span>
        )}

        {/* 头像选择：AI logo 库 */}
        <fieldset className="flex flex-col gap-2">
          <legend className="text-xs font-medium text-secondary-label">
            头像{draft.avatar === null ? '（当前：默认）' : ''}
          </legend>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              aria-label="使用默认头像"
              onClick={() => onDraftChange({ ...draft, avatar: null })}
              className={cn(
                'rounded-full border p-0.5 transition-colors duration-200 ease-spring',
                draft.avatar === null
                  ? 'border-(--color-tint) bg-(--color-tint)/8'
                  : 'border-transparent hover:border-separator',
              )}
            >
              <Avatar
                src={creating ? undefined : defaultAvatarSrc}
                fallback="默"
                alt="默认头像"
                size="sm"
              />
            </button>
            {AVATAR_CHOICES.map((choice) => (
              <button
                key={choice.src}
                type="button"
                aria-label={`头像 ${choice.label}`}
                onClick={() => onDraftChange({ ...draft, avatar: choice.src })}
                className={cn(
                  'rounded-full border p-0.5 transition-colors duration-200 ease-spring',
                  draft.avatar === choice.src
                    ? 'border-(--color-tint) bg-(--color-tint)/8'
                    : 'border-transparent hover:border-separator',
                )}
              >
                <Avatar
                  src={choice.src}
                  alt={choice.label}
                  fallback={choice.label.slice(-1)}
                  size="sm"
                />
              </button>
            ))}
          </div>
        </fieldset>

        <SegmentedControl
          aria-label="供应商"
          value={draft.provider}
          onChange={(provider) => onDraftChange({ ...draft, provider })}
          options={[
            { value: 'auto', label: '跟随全局' },
            { value: 'openai', label: 'OpenAI 兼容' },
            { value: 'anthropic', label: 'Anthropic' },
          ]}
        />
        {(
          [
            ['OpenAI 兼容协议', draft.openai, baseline?.openai, 'openai'],
            ['Anthropic 协议', draft.anthropic, baseline?.anthropic, 'anthropic'],
          ] as const
        ).map(([title, providerDraft, providerBaseline, key]) => (
          <fieldset key={key} className="flex flex-col gap-2">
            <legend className="text-xs font-medium text-secondary-label">{title}</legend>
            <Input
              aria-label={`${title} Base URL`}
              placeholder={providerBaseline?.base_url ?? '继承全局'}
              value={providerDraft.baseUrl}
              onChange={(event) =>
                onDraftChange({
                  ...draft,
                  [key]: { ...providerDraft, baseUrl: event.target.value },
                })
              }
            />
            <Input
              aria-label={`${title} 模型`}
              placeholder={providerBaseline?.model ?? '继承全局'}
              value={providerDraft.model}
              onChange={(event) =>
                onDraftChange({ ...draft, [key]: { ...providerDraft, model: event.target.value } })
              }
            />
            <Input
              type="password"
              aria-label={`${title} API Key`}
              placeholder={
                providerBaseline?.has_key
                  ? `已配置 ${providerBaseline.api_key_masked}，留空沿用`
                  : '未配置，留空继承全局'
              }
              value={providerDraft.apiKey}
              onChange={(event) =>
                onDraftChange({ ...draft, [key]: { ...providerDraft, apiKey: event.target.value } })
              }
            />
          </fieldset>
        ))}
        <label className="flex flex-col gap-1.5">
          <span className="text-xs font-medium text-secondary-label">
            Max Tokens（留空继承 {baseline?.max_tokens ?? '全局'}）
          </span>
          <Input
            type="number"
            min={1}
            max={1_000_000}
            aria-label="Max Tokens"
            value={draft.maxTokens}
            onChange={(event) => onDraftChange({ ...draft, maxTokens: event.target.value })}
          />
        </label>
        <div className="flex items-center justify-between gap-2">
          {!creating ? (
            <Button variant="plain" size="sm" disabled={saving} onClick={onRemove}>
              删除选手
            </Button>
          ) : (
            <span />
          )}
          <Button
            size="sm"
            disabled={saving || (creating ? newName.trim() === '' : !isDraftDirty(draft))}
            onClick={onSave}
          >
            {saving ? '保存中…' : creating ? '创建选手' : '保存覆盖'}
          </Button>
        </div>
      </div>
    </Card>
  );
}
