import { useState } from 'react';
import { ThinkingBlock } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const CONTENT =
  '问题本质是「受控与非受控」的选择：\n\n简单场景（提示气泡）不需要外部干预，用非受控 + autoCollapse 最省事；\n\n复杂场景（多面板联动、外部折叠按钮）才需要 open 受控。\n\n结论：默认非受控，必要时再受控。';

/** 点击回看：结束态下点击标题展开 / 收起，onToggle 回调记录状态 */
export default function ThinkingReview() {
  const [log, setLog] = useState('尚未点击');

  return (
    <Stack gap={3}>
      <ThinkingBlock
        content={CONTENT}
        thinking={false}
        duration={5}
        onToggle={(open) => setLog(open ? '展开（onToggle: true）' : '收起（onToggle: false）')}
      />
      <span className="font-mono text-xs text-tertiary-label">{log}</span>
    </Stack>
  );
}
