import {
  Avatar,
  Button,
  Card,
  Input,
  SegmentedControl,
  Spinner,
  cn,
  type AgentProviderKind,
} from '@agent-packages/shared-ui';
import { useEffect, useState } from 'react';
import { AGENT_POOL } from '../werewolf/boards';
import {
  deletePlayer,
  fetchRoster,
  putPlayerOverride,
  type PlayerOverrideDto,
  type RosterPlayer,
} from './roster';

/** 头像候选：/avatars/ 下的 AI 品牌 logo（换头像与新增选手共用） */
const AVATAR_CHOICES = [
  'claude.png',
  'openai.png',
  'gemini.png',
  'kimi.png',
  'chatglm.png',
  'deepseek.ico',
  'qwen.svg',
  'doubao.png',
  'yuanbao.png',
  'ernie.png',
  'grok.png',
  'mimo.png',
  'minimax.png',
  'mistral.png',
  'llama.png',
].map((file) => ({ label: file.replace(/\.(png|ico|svg)$/, ''), src: `/avatars/${file}` }));

/** 展开编辑的草稿：空串 = 继承全局（不下发）；avatar null = 用默认头像（不下发） */
type ProviderDraft = { baseUrl: string; model: string; apiKey: string };
type PlayerDraft = {
  provider: 'auto' | AgentProviderKind;
  openai: ProviderDraft;
  anthropic: ProviderDraft;
  maxTokens: string;
  avatar: string | null;
};

const EMPTY_DRAFT: PlayerDraft = {
  provider: 'auto',
  openai: { baseUrl: '', model: '', apiKey: '' },
  anthropic: { baseUrl: '', model: '', apiKey: '' },
  maxTokens: '',
  avatar: null,
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
    avatar: player.avatar,
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

/** 草稿 → PUT 请求体：空字段不下发（继承全局 / 用默认头像） */
function overrideFrom(draft: PlayerDraft): PlayerOverrideDto {
  const openaiPatch = patchFrom(draft.openai);
  const anthropicPatch = patchFrom(draft.anthropic);
  return {
    ...(draft.provider === 'auto' ? {} : { provider: draft.provider }),
    ...(Object.keys(openaiPatch).length === 0 ? {} : { openai: openaiPatch }),
    ...(Object.keys(anthropicPatch).length === 0 ? {} : { anthropic: anthropicPatch }),
    ...(draft.maxTokens.trim() === '' ? {} : { max_tokens: Number(draft.maxTokens) }),
    ...(draft.avatar === null ? {} : { avatar: draft.avatar }),
  };
}

function isDraftDirty(draft: PlayerDraft): boolean {
  return (
    draft.provider !== 'auto' ||
    draft.maxTokens.trim() !== '' ||
    draft.avatar !== null ||
    draft.openai.baseUrl.trim() !== '' ||
    draft.openai.model.trim() !== '' ||
    draft.openai.apiKey.trim() !== '' ||
    draft.anthropic.baseUrl.trim() !== '' ||
    draft.anthropic.model.trim() !== '' ||
    draft.anthropic.apiKey.trim() !== ''
  );
}

/** 头像来源：选手自定义 avatar 优先，内置选手回落 AGENT_POOL 静态资源 */
function avatarSrcOf(player: RosterPlayer): string | undefined {
  return player.avatar ?? AGENT_POOL.find((agent) => agent.name === player.name)?.avatarSrc;
}

/** 当前生效的模型名（chip 副标题） */
function modelLabel(player: RosterPlayer): string {
  const view =
    player.effective.provider === 'openai' ? player.effective.openai : player.effective.anthropic;
  return view.model;
}

/**
 * 选手模型管理（无页面 chrome 的纯内容组件）：选手以流式 chip 铺满宽度呈现，
 * 点选展开编辑；「+ 新增选手」可自建选手（PUT 名单外名字自动入库），头像从 AI logo 库挑选。
 * 未覆盖的字段继承全局 AI 设置；只存差异，全空 = 恢复跟随全局。
 * 供设置 Modal 内嵌与独立页共用。
 */
export function RosterManager() {
  const [players, setPlayers] = useState<RosterPlayer[] | null>(null);
  /** 展开编辑的选手名；'__new__' = 新增模式 */
  const [expanded, setExpanded] = useState<string | null>(null);
  const [draft, setDraft] = useState<PlayerDraft>(EMPTY_DRAFT);
  /** 新增模式的选手名 */
  const [newName, setNewName] = useState('');
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

  const creating = expanded === '__new__';

  function select(player: RosterPlayer) {
    if (expanded === player.name) {
      setExpanded(null);
      return;
    }
    setExpanded(player.name);
    setDraft(draftFrom(player));
    setError(null);
  }

  function startCreating() {
    setExpanded('__new__');
    setDraft(EMPTY_DRAFT);
    setNewName('');
    setError(null);
  }

  /** 全局基线（新增选手的占位提示用）：所有选手未覆盖时 effective 即全局 */
  const globalBaseline = players?.[0]?.effective;

  async function save() {
    const name = creating ? newName.trim() : (expanded ?? '');
    if (name === '') {
      setError('选手名不能为空');
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await putPlayerOverride(name, overrideFrom(draft));
      const roster = await fetchRoster();
      setPlayers(roster);
      setExpanded(roster.some((player) => player.name === name) ? name : null);
      setDraft(EMPTY_DRAFT);
      setNewName('');
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : '保存失败');
    } finally {
      setSaving(false);
    }
  }

  async function remove(player: RosterPlayer) {
    setSaving(true);
    setError(null);
    try {
      setPlayers(await deletePlayer(player.name));
      setExpanded(null);
      setDraft(EMPTY_DRAFT);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : '删除失败');
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="flex w-full flex-col gap-4">
      <p className="text-sm leading-relaxed text-secondary-label">
        给每位选手单独配置模型：未填写的字段继承全局 AI 设置，只存差异。「+
        新增选手」可加入自定义选手，头像从 AI logo 库挑选。
      </p>
      {error ? (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : null}
      {!players ? (
        <div className="flex items-center justify-center gap-2 py-10 text-sm text-secondary-label">
          <Spinner size="sm" /> 加载中…
        </div>
      ) : (
        <>
          {/* 选手流：流式 chip 铺满宽度，点选展开编辑 */}
          <div className="flex w-full flex-wrap gap-2.5" role="list" aria-label="选手列表">
            {players.map((player) => {
              const open = expanded === player.name;
              return (
                <button
                  key={player.name}
                  type="button"
                  role="listitem"
                  aria-expanded={open}
                  onClick={() => select(player)}
                  className={cn(
                    'flex max-w-full items-center gap-2 rounded-full border py-1.5 pr-3.5 pl-1.5 text-left transition-colors duration-200 ease-spring',
                    open
                      ? 'border-(--color-tint) bg-(--color-tint)/8 text-label'
                      : 'border-separator bg-secondary-grouped-background text-secondary-label hover:border-(--color-tint) hover:text-label',
                  )}
                >
                  <Avatar
                    src={avatarSrcOf(player)}
                    alt={player.name}
                    fallback={player.name.slice(-1)}
                    size="xs"
                  />
                  <span className="flex min-w-0 flex-col items-start leading-tight">
                    <span className="max-w-36 truncate text-sm font-medium">{player.name}</span>
                    <span className="max-w-36 truncate text-xs text-tertiary-label">
                      {modelLabel(player)}
                    </span>
                  </span>
                  {player.has_override ? (
                    <span
                      aria-label="已单独配置"
                      className="size-1.5 shrink-0 rounded-full bg-(--color-tint)"
                    />
                  ) : null}
                </button>
              );
            })}
            {/* 新增选手：虚线 chip */}
            <button
              type="button"
              aria-expanded={creating}
              onClick={creating ? () => setExpanded(null) : startCreating}
              className={cn(
                'flex items-center gap-1.5 rounded-full border border-dashed px-4 py-2.5 text-sm transition-colors duration-200 ease-spring',
                creating
                  ? 'border-(--color-tint) bg-(--color-tint)/8 text-label'
                  : 'border-separator text-secondary-label hover:border-(--color-tint) hover:text-label',
              )}
            >
              <span aria-hidden className="text-base leading-none">
                +
              </span>
              新增选手
            </button>
          </div>

          {/* 编辑卡：仅展开时渲染 */}
          {expanded !== null ? (
            <Card className="w-full max-w-3xl">
              <div
                aria-label={creating ? '新增选手' : '选手覆盖配置'}
                className="flex flex-col gap-3.5"
              >
                {creating ? (
                  <label className="flex flex-col gap-1.5" htmlFor="new-player-name">
                    <span className="text-xs font-medium text-secondary-label">选手名</span>
                    <Input
                      id="new-player-name"
                      placeholder="例如：DeepSeek R2"
                      value={newName}
                      onChange={(event) => setNewName(event.target.value)}
                    />
                  </label>
                ) : (
                  <span className="text-base font-semibold text-label">{expanded}</span>
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
                      onClick={() => setDraft({ ...draft, avatar: null })}
                      className={cn(
                        'rounded-full border p-0.5 transition-colors duration-200 ease-spring',
                        draft.avatar === null
                          ? 'border-(--color-tint) bg-(--color-tint)/8'
                          : 'border-transparent hover:border-separator',
                      )}
                    >
                      <Avatar
                        src={
                          creating
                            ? undefined
                            : avatarSrcOf(players.find((p) => p.name === expanded) ?? players[0])
                        }
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
                        onClick={() => setDraft({ ...draft, avatar: choice.src })}
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
                  onChange={(provider) => setDraft({ ...draft, provider })}
                  options={[
                    { value: 'auto', label: '跟随全局' },
                    { value: 'openai', label: 'OpenAI 兼容' },
                    { value: 'anthropic', label: 'Anthropic' },
                  ]}
                />
                {(
                  [
                    ['OpenAI 兼容协议', draft.openai, globalBaseline?.openai, 'openai'],
                    ['Anthropic 协议', draft.anthropic, globalBaseline?.anthropic, 'anthropic'],
                  ] as const
                ).map(([title, providerDraft, baseline, key]) => (
                  <fieldset key={key} className="flex flex-col gap-2">
                    <legend className="text-xs font-medium text-secondary-label">{title}</legend>
                    <Input
                      aria-label={`${title} Base URL`}
                      placeholder={baseline?.base_url ?? '继承全局'}
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
                      placeholder={baseline?.model ?? '继承全局'}
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
                        baseline?.has_key
                          ? `已配置 ${baseline.api_key_masked}，留空沿用`
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
                <label className="flex flex-col gap-1.5">
                  <span className="text-xs font-medium text-secondary-label">
                    Max Tokens（留空继承 {globalBaseline?.max_tokens ?? '全局'}）
                  </span>
                  <Input
                    type="number"
                    min={1}
                    max={1_000_000}
                    aria-label="Max Tokens"
                    value={draft.maxTokens}
                    onChange={(event) => setDraft({ ...draft, maxTokens: event.target.value })}
                  />
                </label>
                <div className="flex items-center justify-between gap-2">
                  {!creating && expanded !== null ? (
                    <Button
                      variant="plain"
                      size="sm"
                      disabled={saving}
                      onClick={() => {
                        const target = players.find((p) => p.name === expanded);
                        if (target) void remove(target);
                      }}
                    >
                      删除选手
                    </Button>
                  ) : (
                    <span />
                  )}
                  <Button
                    size="sm"
                    disabled={saving || (creating ? newName.trim() === '' : !isDraftDirty(draft))}
                    onClick={() => void save()}
                  >
                    {saving ? '保存中…' : creating ? '创建选手' : '保存覆盖'}
                  </Button>
                </div>
              </div>
            </Card>
          ) : null}
        </>
      )}
    </div>
  );
}
