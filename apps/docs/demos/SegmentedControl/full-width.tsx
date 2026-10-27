import { useState } from 'react';
import { SegmentedControl } from '@agent-packages/shared-ui';

const options = [
  { value: 'all', label: '全部' },
  { value: 'unread', label: '未读' },
  { value: 'flagged', label: '已标旗' },
] as const;

type Filter = (typeof options)[number]['value'];

export default function SegmentedFullWidth() {
  const [value, setValue] = useState<Filter>('all');
  return (
    <SegmentedControl
      options={options}
      value={value}
      onChange={setValue}
      className="w-full"
      aria-label="邮件筛选"
    />
  );
}
