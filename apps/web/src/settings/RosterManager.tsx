import { Avatar, Spinner, cn } from '@agent-packages/shared-ui';
import { useEffect, useReducer } from 'react';
import { deletePlayer, fetchRoster, putPlayerOverride } from '../apis/settings';
import type { RosterPlayer } from '../apis/types';
import { PlayerEditor } from './PlayerEditor';
import {
  EMPTY_DRAFT,
  avatarSrcOf,
  draftFrom,
  modelLabel,
  overrideFrom,
  type PlayerDraft,
} from './rosterDraft';

/** 展开编辑的选手名；'__new__' = 新增模式 */
const NEW_KEY = '__new__';

type RosterState = {
  /** null = 名单加载中 */
  players: RosterPlayer[] | null;
  expanded: string | null;
  draft: PlayerDraft;
  newName: string;
  error: string | null;
  saving: boolean;
};

type RosterAction =
  | { type: 'loaded'; players: RosterPlayer[] }
  | { type: 'failed'; message: string }
  | { type: 'select'; player: RosterPlayer }
  | { type: 'collapse' }
  | { type: 'startCreating' }
  | { type: 'setSaving'; saving: boolean }
  | { type: 'saved'; players: RosterPlayer[]; kept: string | null }
  | { type: 'setDraft'; draft: PlayerDraft }
  | { type: 'setNewName'; value: string };

function reducer(state: RosterState, action: RosterAction): RosterState {
  switch (action.type) {
    case 'loaded':
      return { ...state, players: action.players };
    case 'failed':
      return { ...state, error: action.message };
    case 'select':
      // 点已展开的选手 = 收起
      if (state.expanded === action.player.name) return { ...state, expanded: null };
      return {
        ...state,
        expanded: action.player.name,
        draft: draftFrom(action.player),
        error: null,
      };
    case 'collapse':
      return { ...state, expanded: null };
    case 'startCreating':
      return { ...state, expanded: NEW_KEY, draft: EMPTY_DRAFT, newName: '', error: null };
    case 'setSaving':
      return { ...state, saving: action.saving, error: action.saving ? null : state.error };
    case 'saved':
      return {
        ...state,
        players: action.players,
        expanded: action.kept,
        draft: EMPTY_DRAFT,
        newName: '',
        saving: false,
      };
    case 'setDraft':
      return { ...state, draft: action.draft };
    case 'setNewName':
      return { ...state, newName: action.value };
  }
}

const INITIAL: RosterState = {
  players: null,
  expanded: null,
  draft: EMPTY_DRAFT,
  newName: '',
  error: null,
  saving: false,
};

/**
 * 选手模型管理（无页面 chrome 的纯内容组件）：选手以流式 chip 铺满宽度呈现，
 * 点选展开编辑；「+ 新增选手」可自建选手（PUT 名单外名字自动入库），头像从 AI logo 库挑选。
 * 未覆盖的字段继承全局 AI 设置；只存差异，全空 = 恢复跟随全局。
 * 状态机（6 项）聚合在 useReducer；编辑卡抽离至 PlayerEditor，草稿模型在 rosterDraft。
 */
export function RosterManager() {
  const [state, dispatch] = useReducer(reducer, INITIAL);
  const { players, expanded, draft, newName, error, saving } = state;

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const roster = await fetchRoster();
        if (!cancelled) dispatch({ type: 'loaded', players: roster });
      } catch (err: unknown) {
        if (!cancelled)
          dispatch({
            type: 'failed',
            message: err instanceof Error ? err.message : '选手配置加载失败',
          });
      }
    }
    void load();
    return () => {
      cancelled = true;
    };
  }, []);

  const creating = expanded === NEW_KEY;
  /** 全局基线（新增选手的占位提示用）：所有选手未覆盖时 effective 即全局 */
  const globalBaseline = players?.[0]?.effective;
  const editingPlayer = creating ? undefined : players?.find((p) => p.name === expanded);

  async function save() {
    const name = creating ? newName.trim() : (expanded ?? '');
    if (name === '') {
      dispatch({ type: 'failed', message: '选手名不能为空' });
      return;
    }
    dispatch({ type: 'setSaving', saving: true });
    try {
      await putPlayerOverride(name, overrideFrom(draft));
      const roster = await fetchRoster();
      dispatch({
        type: 'saved',
        players: roster,
        kept: roster.some((p) => p.name === name) ? name : null,
      });
    } catch (err: unknown) {
      dispatch({
        type: 'failed',
        message: err instanceof Error ? err.message : '保存失败',
      });
      dispatch({ type: 'setSaving', saving: false });
    }
  }

  async function remove(player: RosterPlayer) {
    dispatch({ type: 'setSaving', saving: true });
    try {
      dispatch({ type: 'saved', players: await deletePlayer(player.name), kept: null });
    } catch (err: unknown) {
      dispatch({ type: 'failed', message: err instanceof Error ? err.message : '删除失败' });
      dispatch({ type: 'setSaving', saving: false });
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
          {/* 选手流：流式 chip 铺满宽度，点选展开编辑（button 自带语义，不另套 list role） */}
          <div className="flex w-full flex-wrap gap-2.5" aria-label="选手列表">
            {players.map((player) => {
              const open = expanded === player.name;
              return (
                <button
                  key={player.name}
                  type="button"
                  aria-expanded={open}
                  onClick={() => dispatch({ type: 'select', player })}
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
              onClick={
                creating
                  ? () => dispatch({ type: 'collapse' })
                  : () => dispatch({ type: 'startCreating' })
              }
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
            <PlayerEditor
              creating={creating}
              name={expanded}
              newName={newName}
              onNewNameChange={(value) => dispatch({ type: 'setNewName', value })}
              draft={draft}
              onDraftChange={(next) => dispatch({ type: 'setDraft', draft: next })}
              defaultAvatarSrc={creating ? undefined : avatarSrcOf(editingPlayer ?? players[0])}
              baseline={globalBaseline}
              saving={saving}
              onSave={() => void save()}
              onRemove={() => {
                if (editingPlayer) void remove(editingPlayer);
              }}
            />
          ) : null}
        </>
      )}
    </div>
  );
}
