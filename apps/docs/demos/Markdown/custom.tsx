import { Markdown, type UseMarkdownOptions } from '@agent-packages/shared-ui';

const DOC =
  '访问 [官方文档](https://example.com) 了解更多，或阅读 > 引用块的自定义样式演示。'.replace(
    ' > ',
    '\n\n> ',
  );

const options: UseMarkdownOptions = {
  components: {
    a: ({ children, href }) => (
      <a
        href={href}
        target="_blank"
        rel="noreferrer"
        className="rounded bg-tint/10 px-1 font-medium text-tint no-underline"
      >
        {children}
      </a>
    ),
    blockquote: ({ children }) => (
      <div className="my-3 rounded-xl bg-tint/5 p-3 text-xs text-secondary-label">{children}</div>
    ),
  },
};

export default function MarkdownCustom() {
  return (
    <div className="rounded-xl border border-separator bg-background p-4">
      <Markdown text={DOC} options={options} />
    </div>
  );
}
