import { useState } from 'react';
import { SegmentedControl } from '@agent-packages/shared-ui';

const options = [
  { value: 'list', label: '列表' },
  { value: 'card', label: '卡片' },
  { value: 'grid', label: '网格' },
] as const;

type View = (typeof options)[number]['value'];

export default function SegmentedContent() {
  const [view, setView] = useState<View>('list');
  return (
    <div className="flex flex-col gap-3">
      <SegmentedControl options={options} value={view} onChange={setView} aria-label="视图模式" />
      <div className="rounded-xl bg-tertiary-fill p-3 text-xs text-secondary-label">
        {view === 'list' && '列表视图：单列纵向排布，适合文字密集内容'}
        {view === 'card' && '卡片视图：两列网格，适合图文混排'}
        {view === 'grid' && '网格视图：三列缩略图，适合图片浏览'}
      </div>
    </div>
  );
}
