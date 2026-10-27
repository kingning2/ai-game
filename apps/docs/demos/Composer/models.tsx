import { useState } from 'react';
import { Composer } from '@agent-packages/shared-ui';

const models = [
  { id: 'seed-5-pro', name: 'Seed 5 Pro' },
  { id: 'seed-5-lite', name: 'Seed 5 Lite', badge: '默认' },
  { id: 'seed-4-mini', name: 'Seed 4 Mini' },
];

export default function ComposerModels() {
  const [value, setValue] = useState('');
  const [model, setModel] = useState<string | undefined>('seed-5-lite');
  return (
    <Composer
      value={value}
      onChange={setValue}
      models={models}
      model={model}
      onModelChange={setModel}
      onSend={() => setValue('')}
    />
  );
}
