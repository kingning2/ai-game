import { useEffect, useState } from 'react';
import { Progress } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

/** 模拟上传：进度匀速涨到 100 后停 1.2s，归零重来 */
export default function ProgressDynamic() {
  const [value, setValue] = useState(0);

  useEffect(() => {
    const timer = setInterval(() => {
      setValue((v) => {
        if (v >= 100) return 0;
        return Math.min(100, v + 2);
      });
    }, 80);
    return () => clearInterval(timer);
  }, []);

  return (
    <Stack gap={2}>
      <Progress value={value} />
      <span className="font-mono text-xs tabular-nums text-tertiary-label">上传中… {value}%</span>
    </Stack>
  );
}
