import { useState, type ReactNode } from 'react';
import { cn } from '../cn';
import { Spinner } from '../Spinner';

/**
 * WebSearch 联网搜索卡片 —— 展示 Agent 联网搜索过程与结果。
 *
 * 数据模型对齐 pi 的 web_search 工具：一次调用可含多条查询（queries[]，多角度覆盖），
 * 每条查询独立经历 searching → done，产出 AI 合成答案 + 来源引用列表。
 * 组件零业务逻辑：查询进度、答案流式文本、来源列表都由调用方驱动（与 ToolExecution 同约定）。
 */

export type WebSearchSource = {
  /** 来源标题 */
  title: string;
  /** 来源链接 */
  url: string;
  /** 摘要片段（两行截断） */
  snippet?: string;
};

export type WebSearchQuery = {
  /** 查询词 */
  query: string;
  /** 单条查询状态：searching 检索中 / done 已完成 */
  status: 'searching' | 'done';
  /** AI 合成答案（可流式追加） */
  answer?: string;
  /** 来源引用列表 */
  sources: WebSearchSource[];
  /** 该条查询耗时，结束后由调用方回填 */
  durationMs?: number;
};

export type WebSearchProps = {
  /** 查询数组（多角度），逐条 push / 原位更新即可驱动整个卡片 */
  queries: WebSearchQuery[];
  /** 提供方徽标文字，如 'openai' / 'brave' */
  provider?: string;
  title?: string;
  /** 来源点击回调；不传则渲染为纯文本行 */
  onOpenSource?: (url: string) => void;
  className?: string;
};

const globeIcon = (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-4"
    aria-hidden
  >
    <circle cx="12" cy="12" r="10" />
    <path d="M2 12h20" />
    <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
  </svg>
);

const searchIcon = (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-3.5"
    aria-hidden
  >
    <circle cx="11" cy="11" r="8" />
    <path d="m21 21-4.3-4.3" />
  </svg>
);

const chevronIcon = (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.5"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="size-4 transition-transform duration-200 ease-out"
    aria-hidden
  >
    <path d="m6 9 6 6 6-6" />
  </svg>
);

/** 从 url 提取展示域名（www. 前缀去掉），解析失败原样返回 */
function hostname(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, '');
  } catch {
    return url;
  }
}

function SourceRow({
  index,
  source,
  onOpen,
}: {
  index: number;
  source: WebSearchSource;
  onOpen?: (url: string) => void;
}) {
  const body: ReactNode = (
    <>
      <span className="mt-0.5 flex h-4 min-w-4 shrink-0 items-center justify-center rounded bg-tint/10 px-1 text-[10px] font-semibold text-tint tabular-nums">
        {index}
      </span>
      <span className="min-w-0 flex-1">
        <span className="flex items-baseline gap-1.5">
          <span className="truncate text-[13px] font-medium text-label">{source.title}</span>
          <span className="shrink-0 text-[11px] text-tertiary-label">{hostname(source.url)}</span>
        </span>
        {source.snippet && (
          <span className="mt-0.5 line-clamp-2 block text-xs leading-relaxed text-secondary-label">
            {source.snippet}
          </span>
        )}
      </span>
    </>
  );

  if (!onOpen) {
    return <div className="flex items-start gap-2 py-1">{body}</div>;
  }
  return (
    <button
      type="button"
      onClick={() => onOpen(source.url)}
      className="flex w-full items-start gap-2 rounded-lg py-1 text-start transition-colors duration-150 ease-out hover:bg-tertiary-fill/60"
    >
      {body}
    </button>
  );
}

export function WebSearch({
  queries,
  provider,
  title = '联网搜索',
  onOpenSource,
  className,
}: WebSearchProps) {
  const [collapsed, setCollapsed] = useState(false);

  const doneCount = queries.filter((q) => q.status === 'done').length;
  const searching = queries.some((q) => q.status === 'searching');
  const totalSources = queries.reduce((n, q) => n + q.sources.length, 0);
  const totalMs = queries.reduce((n, q) => n + (q.durationMs ?? 0), 0);

  const statusText = searching
    ? '正在搜索'
    : `${totalSources} 个来源${totalMs > 0 ? ` · ${(totalMs / 1000).toFixed(1)}s` : ''}`;

  return (
    <div className={cn('w-full rounded-xl bg-secondary-grouped-background shadow-card', className)}>
      {/* 头部：图标 + 标题 + provider 徽标 + 状态 + 折叠 */}
      <button
        type="button"
        onClick={() => setCollapsed((c) => !c)}
        aria-expanded={!collapsed}
        className="flex w-full items-center gap-2.5 rounded-xl px-3.5 py-2.5 text-start transition-colors duration-150 ease-out hover:bg-tertiary-fill/50"
      >
        <span
          className={cn(
            'flex size-7 shrink-0 items-center justify-center rounded-lg',
            searching ? 'bg-tint text-white' : 'bg-tint/10 text-tint',
          )}
        >
          {globeIcon}
        </span>
        <span className="min-w-0 flex-1 truncate text-sm font-semibold text-label">{title}</span>
        {provider && (
          <span className="shrink-0 rounded-full bg-tertiary-fill px-2 py-0.5 text-[11px] text-secondary-label">
            {provider}
          </span>
        )}
        <span className="flex shrink-0 items-center gap-1.5 text-xs text-secondary-label tabular-nums">
          {searching && <Spinner size="sm" />}
          {doneCount < queries.length && queries.length > 0
            ? `${doneCount}/${queries.length} · `
            : ''}
          {statusText}
        </span>
        <span className={cn('shrink-0 text-tertiary-label', collapsed && '-rotate-90')}>
          {chevronIcon}
        </span>
      </button>

      {/* 折叠体：grid-rows 0fr→1fr 纯 CSS 过渡（与 ToolExecution 同一手法） */}
      <div
        className={cn(
          'grid transition-[grid-template-rows] duration-300 ease-out',
          collapsed ? 'grid-rows-[0fr]' : 'grid-rows-[1fr]',
        )}
      >
        <div className="overflow-hidden">
          <div className="flex flex-col gap-3 border-t border-separator px-3.5 py-3">
            {queries.length === 0 && (
              <p className="py-1 text-xs text-tertiary-label">正在准备搜索…</p>
            )}

            {queries.map((q, qi) => (
              <div key={`${qi}-${q.query}`} className="flex flex-col gap-1.5">
                {/* 查询行 */}
                <div className="flex items-center gap-2">
                  {q.status === 'searching' ? (
                    <Spinner size="sm" />
                  ) : (
                    <span className="flex size-4 shrink-0 items-center justify-center text-tertiary-label">
                      {searchIcon}
                    </span>
                  )}
                  <span
                    className={cn(
                      'min-w-0 flex-1 truncate text-[13px]',
                      q.status === 'done' ? 'text-label' : 'text-secondary-label',
                    )}
                  >
                    “{q.query}”
                  </span>
                  <span className="shrink-0 text-[11px] text-tertiary-label tabular-nums">
                    {q.status === 'searching'
                      ? '检索中…'
                      : `${q.sources.length} 个来源${q.durationMs != null ? ` · ${(q.durationMs / 1000).toFixed(1)}s` : ''}`}
                  </span>
                </div>

                {/* 答案（流式文本由调用方追加） */}
                {q.answer && (
                  <p className="ps-6 text-[13px] leading-relaxed whitespace-pre-wrap text-label">
                    {q.answer}
                  </p>
                )}

                {/* 来源列表 */}
                {q.sources.length > 0 && (
                  <div className="flex flex-col divide-y divide-separator/60 ps-6">
                    {q.sources.map((s, si) => (
                      <SourceRow
                        key={`${si}-${s.url}`}
                        index={si + 1}
                        source={s}
                        onOpen={onOpenSource}
                      />
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
