import { useState } from 'react';
import { Composer, type ComposerAttachment } from '@agent-packages/shared-ui';

const svgUri = (from: string, to: string) =>
  `data:image/svg+xml;utf8,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="${from}"/><stop offset="1" stop-color="${to}"/></linearGradient></defs><rect width="64" height="64" rx="14" fill="url(#g)"/><circle cx="22" cy="22" r="7" fill="white" opacity="0.85"/><path d="M10 52l16-16 10 10 9-9 11 11" stroke="white" stroke-width="3.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
  )}`;

export default function ComposerAttachments() {
  const [value, setValue] = useState('这两张图的配色帮我对比一下');
  const [attachments, setAttachments] = useState<ComposerAttachment[]>([
    { id: 'a1', url: svgUri('#5B8DEF', '#7ED0FF'), name: 'screenshot-1.svg' },
    { id: 'a2', url: svgUri('#F97362', '#FFC46B'), name: 'screenshot-2.svg' },
  ]);
  return (
    <Composer
      value={value}
      onChange={setValue}
      attachments={attachments}
      onRemoveAttachment={(id) => setAttachments((prev) => prev.filter((a) => a.id !== id))}
      onSend={() => setValue('')}
    />
  );
}
