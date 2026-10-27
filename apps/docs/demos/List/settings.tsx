import { useState } from 'react';
import { List, ListItem, Switch, VStack } from '@agent-packages/shared-ui';

function ToggleRow({ label, defaultOn }: { label: string; defaultOn: boolean }) {
  const [on, setOn] = useState(defaultOn);
  return (
    <ListItem
      title={label}
      trailing={<Switch checked={on} onCheckedChange={setOn} aria-label={label} />}
    />
  );
}

export default function ListSettings() {
  return (
    <VStack gap={2} className="max-w-md">
      <List>
        <ToggleRow label="推送通知" defaultOn />
        <ToggleRow label="深色模式跟随系统" defaultOn />
        <ToggleRow label="省流模式" defaultOn={false} />
      </List>
      <span className="text-xs text-tertiary-label">行尾放控件是 iOS 设置页的标准布局</span>
    </VStack>
  );
}
