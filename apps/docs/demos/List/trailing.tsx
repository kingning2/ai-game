import { Badge, List, ListItem } from '@agent-packages/shared-ui';

export default function ListTrailing() {
  return (
    <List className="max-w-md">
      <ListItem
        title="存储空间"
        subtitle="已使用 128 GB"
        trailing={<span className="text-xs">64%</span>}
      />
      <ListItem
        title="软件更新"
        subtitle="1.5.0 可用"
        trailing={<Badge tone="blue">新版本</Badge>}
      />
      <ListItem title="版本" trailing={<span className="text-xs">1.4.0</span>} />
    </List>
  );
}
