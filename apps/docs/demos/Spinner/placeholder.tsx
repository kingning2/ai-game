import { useEffect, useState } from 'react';
import { Spinner } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

/** 列表加载占位：spinner + 弱色文字是最轻量的加载态 */
export default function SpinnerPlaceholder() {
  const [dots, setDots] = useState('');

  useEffect(() => {
    const timer = setInterval(() => {
      setDots((d) => (d.length >= 3 ? '' : d + '.'));
    }, 400);
    return () => clearInterval(timer);
  }, []);

  return (
    <Stack direction="row" gap={2} align="center">
      <Spinner size="sm" />
      <span className="text-sm text-tertiary-label">正在加载会话记录{dots}</span>
    </Stack>
  );
}
