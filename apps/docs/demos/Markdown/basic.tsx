import { Markdown } from '@agent-packages/shared-ui';

const DOC = `## 发布说明

**v2.4.0** 已发布，本次更新包含 *界面刷新* 与多项稳定性修复。

- 全新的毛玻璃导航栏
- 思考过程面板支持点击回看
- 修复深色模式下表格边框丢失的问题

详情见 [更新日志](https://example.com/changelog)。`;

export default function MarkdownBasic() {
  return (
    <div className="rounded-xl border border-separator bg-background p-4">
      <Markdown text={DOC} />
    </div>
  );
}
