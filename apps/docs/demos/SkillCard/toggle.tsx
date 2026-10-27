import { useState } from 'react';
import { SkillCard, Stack } from '@agent-packages/shared-ui';

/** 启用开关：传入 onToggle 即显示 Switch，启用状态由调用方 useState 驱动 */
export default function SkillCardToggle() {
  const [enabled, setEnabled] = useState({ billing: true, gen: false });

  return (
    <Stack gap={2.5}>
      <SkillCard
        name="arkcli-billing"
        description="查询火山引擎 ARK 拆分账单明细（结算金额、Token 用量计费）"
        tags={['计费', '查询']}
        enabled={enabled.billing}
        onToggle={(v) => setEnabled((s) => ({ ...s, billing: v }))}
      />
      <SkillCard
        name="arkcli-gen"
        description="火山方舟图片/视频生成入口：图片同步返回，视频异步轮询"
        tags={['生成']}
        enabled={enabled.gen}
        onToggle={(v) => setEnabled((s) => ({ ...s, gen: v }))}
      />
      <span className="text-xs text-tertiary-label">
        切换开关查看状态流转；组件本身不持有开关状态
      </span>
    </Stack>
  );
}
