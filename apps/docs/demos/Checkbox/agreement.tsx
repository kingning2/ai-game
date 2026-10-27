import { useState } from 'react';
import { Button, Checkbox, Stack } from '@agent-packages/shared-ui';

export default function CheckboxAgreement() {
  const [agreed, setAgreed] = useState(false);
  return (
    <Stack gap={3}>
      <div className="flex items-start gap-2.5">
        <Checkbox
          checked={agreed}
          onCheckedChange={setAgreed}
          className="mt-0.5"
          aria-label="同意协议"
        />
        <span className="text-sm text-label">
          我已阅读并同意
          <span className="text-tint">《用户协议》</span>与
          <span className="text-tint">《隐私政策》</span>
        </span>
      </div>
      <Button variant="solid" disabled={!agreed}>
        注册
      </Button>
    </Stack>
  );
}
