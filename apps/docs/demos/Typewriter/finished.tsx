import { useState } from 'react';
import { Typewriter } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const TEXT = '已为你整理好本周的会议纪要，重点有三处修改，详见附录。';

export default function TypewriterFinished() {
  const [finished, setFinished] = useState(false);

  return (
    <Stack gap={3}>
      <Typewriter text={TEXT} onFinished={() => setFinished(true)} />
      {finished ? (
        <span className="w-fit rounded-full bg-green/15 px-2.5 py-0.5 text-xs text-green">
          ✓ 回复生成完毕
        </span>
      ) : (
        <span className="text-xs text-tertiary-label">输出中…（onFinished 触发后显示状态）</span>
      )}
    </Stack>
  );
}
