import { Avatar, List, ListItem } from '@agent-packages/shared-ui';

const members = [
  { name: '林小临', desc: '前端工程师 · 设计令牌', letter: '林' },
  { name: '周小雨', desc: '产品设计师 · 交互稿', letter: '周' },
  { name: '王一舟', desc: '后端工程师 · API 网关', letter: '王' },
];

export default function AvatarList() {
  return (
    <List className="max-w-md">
      {members.map((m) => (
        <ListItem
          key={m.name}
          media={<Avatar fallback={m.letter} size="sm" />}
          title={m.name}
          subtitle={m.desc}
        />
      ))}
    </List>
  );
}
