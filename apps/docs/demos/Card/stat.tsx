import { Card, HStack } from '@agent-packages/shared-ui';

const stats = [
  { label: '总请求数', value: '128.4K', delta: '+12.5%', up: true },
  { label: '错误率', value: '0.02%', delta: '-0.01%', up: true },
  { label: 'P99 延迟', value: '231ms', delta: '+18ms', up: false },
];

export default function CardStat() {
  return (
    <HStack gap={3} wrap>
      {stats.map((s) => (
        <Card key={s.label} className="min-w-36 flex-1">
          <span className="text-xs text-secondary-label">{s.label}</span>
          <div className="mt-1 text-xl font-semibold text-label">{s.value}</div>
          <span className={s.up ? 'text-xs text-green' : 'text-xs text-red'}>{s.delta}</span>
        </Card>
      ))}
    </HStack>
  );
}
