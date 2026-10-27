import { useState } from 'react';
import { Stack, ToolExecution, type ToolExecutionData } from '@agent-packages/shared-ui';

const INITIAL: ToolExecutionData[] = [
  {
    id: 'bash-1',
    name: 'bash',
    args: { command: 'pnpm test' },
    status: 'error',
    error: 'command not found: pnpm\nexit code 1',
    durationMs: 640,
  },
  {
    id: 'read-1',
    name: 'read_file',
    args: { path: 'packages/shared-ui/styles' },
    status: 'success',
    result: 'styles/index.css / tokens.css / animations.css',
    durationMs: 310,
  },
  {
    id: 'search-1',
    name: 'web_search',
    args: { query: 'Apple HIG colors' },
    status: 'success',
    result: '共 5 条结果：1. Apple HIG — Colors…',
    durationMs: 820,
  },
];

/** 多工具序列：agent 一次回复中的工具调用流（最新在前）；失败项重试后恢复成功 */
export default function ToolExecutionSequence() {
  const [tools, setTools] = useState(INITIAL);

  const retry = (id: string) => {
    setTools((list) =>
      list.map((e) =>
        e.id === id ? { ...e, status: 'running', error: undefined, durationMs: undefined } : e,
      ),
    );
    window.setTimeout(() => {
      setTools((list) =>
        list.map((e) =>
          e.id === id
            ? { ...e, status: 'success', result: '32 passed (4.2s)', durationMs: 4200 }
            : e,
        ),
      );
    }, 1200);
  };

  return (
    <Stack gap={2}>
      {tools.map((e) => (
        <ToolExecution
          key={e.id}
          execution={e}
          onRetry={e.status === 'error' ? () => retry(e.id) : undefined}
        />
      ))}
    </Stack>
  );
}
