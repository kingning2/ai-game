import { List, ListItem } from '@agent-packages/shared-ui';

export default function ListBasic() {
  return (
    <List className="max-w-md">
      <ListItem title="通用" />
      <ListItem title="隐私与安全性" />
      <ListItem title="通知" />
    </List>
  );
}
