import { useState } from 'react';
import { Button, Stack, ToolExecution, type ToolExecutionData } from '@agent-packages/shared-ui';

const ARGS = { query: 'Apple HIG colors', limit: 5 };
const RESULT = [
  '共 5 条结果：',
  '1. Apple HIG — Colors（developer.apple.com/design）',
  '2. Materials — 毛玻璃层级指南',
  '3. SF Symbols — 图标命名规范',
  '…',
].join('\n');

/** 成功生命周期：点击运行 → running（1.2s）→ success，点行展开参数与输出 */
export default function ToolExecutionBasic() {
  const [execution, setExecution] = useState<ToolExecutionData | null>(null);

  const run = () => {
    const id = `web_search-${Date.now()}`;
    setExecution({ id, name: 'web_search', args: ARGS, status: 'running' });
    window.setTimeout(() => {
      setExecution({
        id,
        name: 'web_search',
        args: ARGS,
        status: 'success',
        result: RESULT,
        durationMs: 820,
      });
    }, 1200);
  };

  return (
    <Stack gap={3}>
      <Button variant="tinted" size="sm" onClick={run}>
        运行 web_search
      </Button>
      {execution ? (
        <ToolExecution execution={execution} />
      ) : (
        <span className="text-xs text-tertiary-label">
          点击按钮模拟一次工具调用：running → 成功；点行展开参数与输出
        </span>
      )}
    </Stack>
  );
}
