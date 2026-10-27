import { WebSearch, type WebSearchQuery } from '@agent-packages/shared-ui';

/** 对齐 pi 的 queries[] 模型：多角度查询，每条独立 searching→done + 合成答案 + 来源 */
const QUERIES: WebSearchQuery[] = [
  {
    query: 'CSS scroll-state 浏览器支持',
    status: 'done',
    answer:
      'scroll-state() 容器查询目前仅 Chromium 133+（2025-02 起）支持，Firefox 与 Safari 均未跟进，全局覆盖率约 71%。建议作为渐进增强使用，配合 JS 兜底。[1]',
    durationMs: 3240,
    sources: [
      {
        title: 'Container scroll-state queries | Can I use',
        url: 'https://caniuse.com/wf-container-scroll-state-queries',
        snippet: 'Chrome 133+, Edge 133+；Firefox 与 Safari 尚未支持该特性。',
      },
      {
        title: 'CSS scroll-state() | Chrome for Developers',
        url: 'https://developer.chrome.com/blog/css-scroll-state-queries',
        snippet: 'Like container queries; but for stuck, snapped, and overflowing queries.',
      },
    ],
  },
  {
    query: 'popover API anchor positioning 现状',
    status: 'done',
    answer:
      'Popover API 已于 2025-01 进入 Baseline（三大引擎可用）；CSS anchor positioning 在 Safari 26 已发布，Firefox 仍在开发中。[1][2]',
    durationMs: 2870,
    sources: [
      {
        title: 'The Popover API is now Baseline Newly available | web.dev',
        url: 'https://web.dev/blog/popover-baseline',
        snippet: 'This web feature is now available in all three major browser engines.',
      },
      {
        title: 'Using CSS anchor positioning - CSS | MDN',
        url: 'https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Anchor_positioning/Using',
        snippet: 'Implicit anchor association via popovertarget and id attributes.',
      },
      {
        title: 'Container scroll-state queries | Can I use',
        url: 'https://caniuse.com/wf-container-scroll-state-queries',
      },
    ],
  },
  {
    query: 'CSS @starting-style 实战示例',
    status: 'searching',
    sources: [],
  },
];

/** 基础用法：多角度查询 + 合成答案 + 来源引用，其中一条仍在检索 */
export default function WebSearchBasic() {
  return (
    <WebSearch
      queries={QUERIES}
      provider="openai"
      onOpenSource={(url) => window.open(url, '_blank')}
    />
  );
}
