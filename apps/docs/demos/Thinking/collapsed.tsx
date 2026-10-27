import { ThinkingBlock } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const CONTENT =
  '第一步：解析用户意图，确认要生成周报的截止日期。\n第二步：检索本周的会议纪要与提交记录。\n第三步：按「进展 / 风险 / 下周计划」三段式组织内容。';

/** 结束态：thinking=false 自动收起，标题显示用时 */
export default function ThinkingCollapsed() {
  return (
    <Stack gap={3}>
      <ThinkingBlock content={CONTENT} thinking={false} duration={12} />
      <span className="text-xs text-tertiary-label">默认收起状态，点击标题可回看完整思维链</span>
    </Stack>
  );
}
