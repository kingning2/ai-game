import { useEffect, useState } from 'react';
import { UsageMeter } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

/** 模拟对话中上下文不断增长的实时占用 */
export default function UsageMeterLive() {
  const [used, setUsed] = useState(60000);

  useEffect(() => {
    const timer = setInterval(() => {
      setUsed((u) => (u >= 196000 ? 60000 : u + 2400));
    }, 120);
    return () => clearInterval(timer);
  }, []);

  return (
    <Stack gap={2} className="w-full max-w-120">
      <UsageMeter used={used} total={200000} label="上下文窗口" />
      <span className="text-xs text-tertiary-label">70% 转橙、90% 转红，超过阈值自动预警</span>
    </Stack>
  );
}
