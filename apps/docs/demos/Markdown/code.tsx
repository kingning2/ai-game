import { Markdown } from '@agent-packages/shared-ui';

const DOC = `### 代码展示

行内代码用 \`useMarkdown(text)\` 即可，块级代码自带描边容器与横向滚动：

\`\`\`ts
const node = useMarkdown('# 你好', {
  components: { a: MyLink }, // 逐元素覆盖
});
\`\`\`

也可以不写 hook，直接 \`<Markdown text={src} />\` 薄壳调用。`;

export default function MarkdownCode() {
  return (
    <div className="rounded-xl border border-separator bg-background p-4">
      <Markdown text={DOC} />
    </div>
  );
}
