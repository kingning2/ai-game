import { useState } from 'react';
import { Composer } from '@agent-packages/shared-ui';

export default function ComposerMaxRows() {
  const [value, setValue] = useState('');
  return (
    <Composer
      value={value}
      onChange={setValue}
      placeholder="试试粘贴一段长文本，超过 3 行后内部滚动"
      maxRows={3}
      onSend={() => setValue('')}
    />
  );
}
