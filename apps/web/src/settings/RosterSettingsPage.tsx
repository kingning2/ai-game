import {
  Avatar,
  Badge,
  Button,
  Card,
  Input,
  List,
  ListItem,
  NavBar,
  SegmentedControl,
  Spinner,
  cn,
  type AgentProviderKind,
} from '@agent-packages/shared-ui';
import { Fragment, useEffect, useState } from 'react';
import { AGENT_POOL } from '../werewolf/boards';
import {
  fetchRoster,
  putPlayerOverride,
  type PlayerOverrideDto,
  type RosterPlayer,
} from './roster';

export type RosterSettingsPageProps = {
  onBack: () => void;
  className?: string;
};

/** 展开编辑的草稿：空串 = 继承全局（不下发） */
type ProviderDraft = { baseUrl: string; model: string; apiKey: string };
type PlayerDraft = {
  provider: 'auto' | AgentProviderKind;
  openai: ProviderDraft;
  anthropic: ProviderDraft;
  maxTokens: string;
};

const EMPTY_DRAFT: PlayerDraft = {
  provider: 'auto',
  openai: { baseUrl: '', model: '', apiKey: '' },
  anthropic: { baseUrl: '', model: '', apiKey: '' },
  maxTokens: '',
};

function draftFrom(player: RosterPlayer): PlayerDraft {
  const player_override = player.player_override;
  return {
    provider: player_override?.provider ?? 'auto',
    openai: {
      baseUrl: player_override?.openai?.base_url ?? '',
      model: player_override?.openai?.model ?? '',
      apiKey: '',
    },
    anthropic: {
      baseUrl: player_override?.anthropic?.base_url ?? '',
      model: player_override?.anthropic?.model ?? '',
      apiKey: '',
    },
    maxTokens: player_override?.max_tokens !== undefined ? String(player_override.max_tokens) : '',
  };
}

/** 单协议草稿 → 请求体补丁：空串字段不下发（继承全局） */
function patchFrom(provider: ProviderDraft) {
  return {
    ...(provider.baseUrl.trim() === '' ? {} : { base_url: provider.baseUrl.trim() }),
    ...(provider.model.trim() === '' ? {} : { model: provider.model.trim() }),
    ...(provider.apiKey.trim() === '' ? {} : { api_key: provider.apiKey.trim() }),
  };
}

/** 草稿 → PUT 请求体：空串字段不下发（继承全局） */
function overrideFrom(draft: PlayerDraft): PlayerOverrideDto {
  const openaiPatch = patchFrom(draft.openai);
  const anthropicPatch = patchFrom(draft.anthropic);
  return {
    ...(draft.provider === 'auto' ? {} : { provider: draft.provider }),
    ...(Object.keys(openaiPatch).length === 0 ? {} : { openai: openaiPatch }),
    ...(Object.keys(anthropicPatch).length === 0 ? {} : { anthropic: anthropicPatch }),
    ...(draft.maxTokens.trim() === '' ? {} : { max_tokens: Number(draft.maxTokens) }),
  };
}

function isDraftDirty(draft: PlayerDraft): boolean {
  return (
    draft.provider !== 'auto' ||
    draft.maxTokens.trim() !== '' ||
    draft.openai.baseUrl.trim() !== '' ||
    draft.openai.model.trim() !== '' ||
    draft.openai.apiKey.trim() !== '' ||
    draft.anthropic.baseUrl.trim() !== '' ||
    draft.anthropic.model.trim() !== '' ||
    draft.anthropic.apiKey.trim() !== ''
  );
}

function avatarOf(name: string): string | undefined {
  return AGENT_POOL.find((agent) => agent.name === name)?.avatarSrc;
}

/**
 * 选手模型配置页：12 名参赛 AI 各自的 provider / 模型 / Key。
 * 未覆盖的字段继承全局 AI 设置（PUT /api/agent/config）；此处只存差异，全空 = 恢复跟随全局。
 */
export function RosterSettingsPage({ onBack, className }: RosterSettingsPageProps) {
  const [players, setPlayers] = useState<RosterPlayer[] | null>(null);
  const [expanded, setExpanded] = useState<string | null>(null);
  const [draft, setDraft] = useState<PlayerDraft>(EMPTY_DRAFT);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const roster = await fetchRoster();
        if (!cancelled) setPlayers(roster);
      } catch (err: unknown) {
        if (!cancelled) setError(err instanceof Error ? err.message : '选手配置加载失败');
      }
    }
    void load();
    return () => {
      cancelled = true;
    };
  }, []);

  function toggle(player: RosterPlayer) {
    if (expanded === player.name) {
      setExpanded(null);
      return;
    }
    setExpanded(player.name);
    setDraft(draftFrom(player));
    setError(null);
  }

  async function save(player: RosterPlayer, override: PlayerOverrideDto) {
    setSaving(true);
    setError(null);
    try {
      await putPlayerOverride(player.name, override);
      setPlayers(await fetchRoster());
      setExpanded(null);
      setDraft(EMPTY_DRAFT);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : '保存失败');
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className={`min-h-dvh bg-secondary-background ${className ?? ''}`}>
      <NavBar
        sticky
        leading={
          <Button variant="plain" size="sm" onClick={onBack}>
            ‹ 返回设置
          </Button>
        }
        title="选手模型"
        trailing={<span className="w-20" aria-hidden />}
      />
      <div className="mx-auto flex max-w-md flex-col gap-3 px-6 py-8">
        <p className="text-xs leading-relaxed text-secondary-label">
          给每位选手单独配置模型：未填写的字段继承全局 AI 设置，只存差异。这里改完只影响对应选手。
        </p>
        {error ? (
          <p className="text-sm text-red" role="alert">
            {error}
          </p>
        ) : null}
        {!players ? (
          <div className="flex items-center justify-center gap-2 py-12 text-sm text-secondary-label">
            <Spinner size="sm" /> 加载中…
          </div>
        ) : (
          <List>
            {players.map((player) => {
              const open = expanded === player.name;
              const view =
                player.effective.provider === 'openai'
                  ? player.effective.openai
                  : player.effective.anthropic;
              return (
                <Fragment key={player.name}>
                  <ListItem
                    media={
                      <Avatar
                        src={avatarOf(player.name)}
                        alt={player.name}
                        fallback={player.name.slice(-1)}
                      />
                    }
                    title={player.name}
                    subtitle={`${view.model} · ${
                      player.effective.provider === 'openai' ? 'OpenAI 兼容' : 'Anthropic'
                    }`}
                    trailing={
                      <>
                        {player.has_override ? <Badge tone="tint">已单独配置</Badge> : null}
                        <span
                          aria-hidden
                          className={cn(
                            'transition-transform duration-200 ease-spring',
                            open && 'rotate-90',
                          )}
                        >
                          ›
                        </span>
                      </>
                    }
                    aria-expanded={open}
                    onClick={() => toggle(player)}
                  />
                  {open ? (
                    <Card
                      aria-label={`${player.name} 的覆盖配置`}
                      className="flex flex-col gap-3 bg-background"
                    >
                      <SegmentedControl
                        aria-label={`${player.name} 的供应商`}
                        value={draft.provider}
                        onChange={(provider) => setDraft({ ...draft, provider })}
                        options={[
                          { value: 'auto', label: '跟随全局' },
                          { value: 'openai', label: 'OpenAI 兼容' },
                          { value: 'anthropic', label: 'Anthropic' },
                        ]}
                      />
                      {(
                        [
                          ['OpenAI 兼容协议', draft.openai, player.effective.openai, 'openai'],
                          [
                            'Anthropic 协议',
                            draft.anthropic,
                            player.effective.anthropic,
                            'anthropic',
                          ],
                        ] as const
                      ).map(([title, providerDraft, effective, key]) => (
                        <fieldset key={key} className="flex flex-col gap-2">
                          <legend className="text-xs font-medium text-secondary-label">
                            {title}
                          </legend>
                          <Input
                            aria-label={`${title} Base URL`}
                            placeholder={effective.base_url}
                            value={providerDraft.baseUrl}
                            onChange={(event) =>
                              setDraft({
                                ...draft,
                                [key]: { ...providerDraft, baseUrl: event.target.value },
                              })
                            }
                          />
                          <Input
                            aria-label={`${title} 模型`}
                            placeholder={effective.model}
                            value={providerDraft.model}
                            onChange={(event) =>
                              setDraft({
                                ...draft,
                                [key]: { ...providerDraft, model: event.target.value },
                              })
                            }
                          />
                          <Input
                            type="password"
                            aria-label={`${title} API Key`}
                            placeholder={
                              effective.has_key
                                ? `已配置 ${effective.api_key_masked}，留空沿用`
                                : '未配置，留空继承全局'
                            }
                            value={providerDraft.apiKey}
                            onChange={(event) =>
                              setDraft({
                                ...draft,
                                [key]: { ...providerDraft, apiKey: event.target.value },
                              })
                            }
                          />
                        </fieldset>
                      ))}
                      <label className="flex flex-col gap-1">
                        <span className="text-xs font-medium text-secondary-label">
                          Max Tokens（留空继承 {player.effective.max_tokens}）
                        </span>
                        <Input
                          type="number"
                          min={1}
                          max={1_000_000}
                          aria-label="Max Tokens"
                          value={draft.maxTokens}
                          onChange={(event) =>
                            setDraft({ ...draft, maxTokens: event.target.value })
                          }
                        />
                      </label>
                      <div className="flex items-center justify-between gap-2">
                        {player.has_override ? (
                          <Button
                            variant="plain"
                            size="sm"
                            disabled={saving}
                            onClick={() => {
                              void save(player, {});
                            }}
                          >
                            恢复跟随全局
                          </Button>
                        ) : (
                          <span />
                        )}
                        <Button
                          size="sm"
                          disabled={saving || !isDraftDirty(draft)}
                          onClick={() => {
                            void save(player, overrideFrom(draft));
                          }}
                        >
                          {saving ? '保存中…' : '保存覆盖'}
                        </Button>
                      </div>
                    </Card>
                  ) : null}
                </Fragment>
              );
            })}
          </List>
        )}
      </div>
    </div>
  );
}
