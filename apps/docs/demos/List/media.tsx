import type { ReactNode } from 'react';
import { List, ListItem } from '@agent-packages/shared-ui';

function IconTile({ color, children }: { color: string; children: ReactNode }) {
  return (
    <span
      className="flex size-8 items-center justify-center rounded-lg text-white"
      style={{ backgroundColor: color }}
    >
      <svg viewBox="0 0 24 24" className="size-4.5" aria-hidden>
        {children}
      </svg>
    </span>
  );
}

const items = [
  {
    title: '收藏',
    subtitle: '12 个已收藏项目',
    color: '#FF9F0A',
    icon: (
      <path
        d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
        fill="currentColor"
      />
    ),
  },
  {
    title: '通知',
    subtitle: '3 条未读提醒',
    color: '#0A84FF',
    icon: (
      <path
        d="M12 22a2.5 2.5 0 0 0 2.45-2h-4.9A2.5 2.5 0 0 0 12 22zm7-6v-5c0-3.1-1.7-5.6-4.5-6.3V4a2.5 2.5 0 0 0-5 0v.7C6.7 5.4 5 7.9 5 11v5l-2 2v1h18v-1l-2-2z"
        fill="currentColor"
      />
    ),
  },
  {
    title: '已完成',
    subtitle: '本周完成 8 项任务',
    color: '#30D158',
    icon: <path d="M9 16.2L4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4L9 16.2z" fill="currentColor" />,
  },
];

export default function ListMedia() {
  return (
    <List className="max-w-md">
      {items.map((it) => (
        <ListItem
          key={it.title}
          media={<IconTile color={it.color}>{it.icon}</IconTile>}
          title={it.title}
          subtitle={it.subtitle}
        />
      ))}
    </List>
  );
}
