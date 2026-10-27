import { useState } from 'react';
import { Button, ErrorBanner } from '@agent-packages/shared-ui';
import type { ApiErrorKind } from '@agent-packages/shared';

/** kind 决定默认文案：与 @agent-packages/shared 的 ApiError.kind 对齐 */
export default function ErrorBannerKinds() {
  const [kind, setKind] = useState<ApiErrorKind>('network');

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap gap-1.5">
        {(['network', 'business', 'parse', 'http', 'abort'] as ApiErrorKind[]).map((k) => (
          <Button
            key={k}
            variant={kind === k ? 'tinted' : 'plain'}
            size="sm"
            onClick={() => setKind(k)}
          >
            {k}
          </Button>
        ))}
      </div>
      <ErrorBanner kind={kind} message="GET /api/v3/usage/stats" onRetry={() => {}} />
    </div>
  );
}
