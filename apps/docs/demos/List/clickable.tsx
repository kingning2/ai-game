import { useState } from 'react';
import { List, ListItem, VStack } from '@agent-packages/shared-ui';

const actions = ['重命名', '移动到…', '删除'];

export default function ListClickable() {
  const [last, setLast] = useState<string | null>(null);
  return (
    <VStack gap={2} className="max-w-md">
      <List>
        {actions.map((a) => (
          <ListItem key={a} title={a} onClick={() => setLast(a)} />
        ))}
      </List>
      <span className="text-xs text-tertiary-label">最近点击：{last ?? '（无）'}</span>
    </VStack>
  );
}
