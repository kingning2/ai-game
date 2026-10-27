import { useState } from 'react';
import { Composer } from '@agent-packages/shared-ui';

export default function ComposerDisabled() {
  const [value, setValue] = useState('整块输入区已禁用');
  return <Composer value={value} onChange={setValue} disabled onSend={() => {}} />;
}
