import { AGENT_POOL } from '../werewolf/boards';
import type { PlayerOverrideDto, RosterPlayer } from '../apis/types';
import type { AgentProviderKind } from '@agent-packages/shared-ui';

/** 头像候选：/avatars/ 下的 AI 品牌 logo（换头像与新增选手共用） */
export const AVATAR_CHOICES = [
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
export type ProviderDraft = { baseUrl: string; model: string; apiKey: string };
export type PlayerDraft = {
  provider: 'auto' | AgentProviderKind;
  openai: ProviderDraft;
  anthropic: ProviderDraft;
  maxTokens: string;
  avatar: string | null;
};

export const EMPTY_DRAFT: PlayerDraft = {
  provider: 'auto',
  openai: { baseUrl: '', model: '', apiKey: '' },
  anthropic: { baseUrl: '', model: '', apiKey: '' },
  maxTokens: '',
  avatar: null,
};

export function draftFrom(player: RosterPlayer): PlayerDraft {
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
export function overrideFrom(draft: PlayerDraft): PlayerOverrideDto {
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

export function isDraftDirty(draft: PlayerDraft): boolean {
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
export function avatarSrcOf(player: RosterPlayer): string | undefined {
  return player.avatar ?? AGENT_POOL.find((agent) => agent.name === player.name)?.avatarSrc;
}

/** 当前生效的模型名（chip 副标题） */
export function modelLabel(player: RosterPlayer): string {
  const view =
    player.effective.provider === 'openai' ? player.effective.openai : player.effective.anthropic;
  return view.model;
}
