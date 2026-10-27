import { Card } from '@agent-packages/shared-ui';

const tiles = [
  { title: '外观', desc: '浅色 / 深色 / 跟随系统' },
  { title: '通知', desc: '免打扰时段与推送策略' },
  { title: '隐私', desc: '数据收集与权限管理' },
  { title: '通用', desc: '语言、地区与日期格式' },
];

export default function CardGrid() {
  return (
    <div className="grid max-w-lg grid-cols-2 gap-3">
      {tiles.map((t) => (
        <Card key={t.title}>
          <span className="text-sm font-medium text-label">{t.title}</span>
          <p className="mt-1 text-xs text-secondary-label">{t.desc}</p>
        </Card>
      ))}
    </div>
  );
}
