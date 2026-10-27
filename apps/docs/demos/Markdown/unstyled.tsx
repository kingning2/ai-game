import { Markdown } from '@agent-packages/shared-ui';

const DOC =
  '## 无样式模式\n\n`unstyled` 跳过内置主题，**完全**使用调用方传入的 components（适合嵌入已有设计系统）。';

export default function MarkdownUnstyled() {
  return (
    <div className="rounded-xl border border-separator bg-background p-4">
      <Markdown
        text={DOC}
        options={{
          unstyled: true,
          className: '[&_h2]:text-lg [&_h2]:font-bold [&_strong]:text-tint',
        }}
      />
    </div>
  );
}
