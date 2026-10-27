import { useEffect, useRef, useState } from 'react';
import { Button, WebSearch, type WebSearchQuery } from '@agent-packages/shared-ui';

const ROUNDS: Array<{
  query: string;
  answer: string;
  sources: WebSearchQuery['sources'];
  ms: number;
}> = [
  {
    query: 'Agent UI 组件库 设计趋势',
    answer:
      'Agent 界面正从聊天窗口走向「计划 + 工具 + 结果」的结构化卡片流，Apple HIG 风格的状态披露是主流。[1][2]',
    sources: [
      {
        title: 'Designing agent interfaces | emilkowalski',
        url: 'https://emilkowalski.com/notes',
        snippet: 'Interruptible, state-driven UI beats fixed keyframe timelines.',
      },
      {
        title: 'AI 组件库实践 | web.dev',
        url: 'https://web.dev/patterns',
        snippet: 'Progressive disclosure for tool results.',
      },
    ],
    ms: 2600,
  },
  {
    query: 'React 流式渲染 最佳实践',
    answer: '流式文本直接绑定受控 state 逐段追加，避免每字符 setState；组件零时序假设。[1]',
    sources: [
      {
        title: 'react.dev | Streaming',
        url: 'https://react.dev/reference/react/streaming',
        snippet: 'Append to controlled state; let CSS handle the rest.',
      },
    ],
    ms: 1900,
  },
];

/** 推进交互：模拟 pi 的搜索过程 —— 逐条 push 查询、答案流式追加、状态推进 */
export default function WebSearchLive() {
  const [queries, setQueries] = useState<WebSearchQuery[]>([]);
  const [round, setRound] = useState(0);
  const [busy, setBusy] = useState(false);
  const timers = useRef<ReturnType<typeof setTimeout>[]>([]);

  useEffect(() => () => timers.current.forEach(clearTimeout), []);

  const run = () => {
    if (busy || round >= ROUNDS.length) return;
    setBusy(true);
    const r = ROUNDS[round];
    const started = performance.now();

    // 1) push searching 查询
    setQueries((qs) => [...qs, { query: r.query, status: 'searching', sources: [] }]);

    // 2) 1s 后开始流式追加答案
    timers.current.push(
      setTimeout(() => {
        const words = r.answer.split('');
        let i = 0;
        const tick = () => {
          i += 3;
          const text = words.slice(0, i).join('');
          setQueries((qs) => qs.map((q, idx) => (idx === round ? { ...q, answer: text } : q)));
          if (i < words.length) {
            timers.current.push(setTimeout(tick, 60));
          } else {
            // 3) 流式完成 → done，回填来源与耗时
            setQueries((qs) =>
              qs.map((q, idx) =>
                idx === round
                  ? {
                      ...q,
                      status: 'done',
                      sources: r.sources,
                      durationMs: performance.now() - started,
                    }
                  : q,
              ),
            );
            setRound((n) => n + 1);
            setBusy(false);
          }
        };
        tick();
      }, 1000),
    );
  };

  return (
    <div className="flex flex-col gap-3">
      <WebSearch
        queries={queries}
        provider="openai"
        onOpenSource={(url) => window.open(url, '_blank')}
      />
      <Button
        variant="tinted"
        size="sm"
        onClick={run}
        disabled={busy || round >= ROUNDS.length}
        className="self-start"
      >
        {busy
          ? '搜索中…'
          : round === 0
            ? '发起搜索'
            : round >= ROUNDS.length
              ? '已完成'
              : '追加查询'}
      </Button>
      <span className="text-xs text-tertiary-label">
        组件零时序假设：push / 原位更新 queries 即可驱动全部状态
      </span>
    </div>
  );
}
