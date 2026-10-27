import { useState } from 'react';
import { SegmentedControl } from '@agent-packages/shared-ui';

const options = [
  { value: 'latest', label: '最新' },
  { value: 'hot', label: '最热' },
  { value: 'recommended', label: '推荐' },
] as const;

type Sort = (typeof options)[number]['value'];

const FEED: Record<Sort, string[]> = {
  latest: ['刚刚发布的新版本', '三分钟前的动态', '一小时前的讨论'],
  hot: ['本周最热话题', '千赞精选回答', '热议中的争议'],
  recommended: ['为你推荐的专栏', '猜你感兴趣的作者', '同好都在看'],
};

export default function SegmentedFilter() {
  const [sort, setSort] = useState<Sort>('latest');
  return (
    <div className="flex flex-col gap-3">
      <SegmentedControl options={options} value={sort} onChange={setSort} aria-label="排序方式" />
      <ul className="m-0 list-none p-0">
        {FEED[sort].map((title) => (
          <li
            key={title}
            className="border-b border-separator py-2 text-sm text-label last:border-b-0"
          >
            {title}
          </li>
        ))}
      </ul>
    </div>
  );
}
