import { SkillCard, Stack } from '@agent-packages/shared-ui';

const SKILLS = [
  {
    name: 'arkcli-billing',
    description:
      '查询火山引擎 ARK 拆分账单明细（结算金额、Token 用量计费），支持按账期月、Endpoint、API Key 过滤',
    tags: ['计费', '查询'],
  },
  {
    name: 'arkcli-chat',
    description: '通过数据面 Responses API 快速对话/推理，支持多模态、流式、多轮与无副作用 dry-run',
    tags: ['对话', '推理'],
  },
  {
    name: 'arkcli-gen',
    description: '火山方舟图片/视频生成入口：图片同步返回，视频异步轮询',
    tags: ['生成'],
  },
  {
    name: 'arkcli-deploy',
    description: '普通创建推理接入点（Endpoint）的统一首选入口，创建成功后自动渲染多语言调用示例',
  },
];

/** 基础用法：技能网格（二维布局才用 grid），纯展示不带开关 */
export default function SkillCardBasic() {
  return (
    <Stack gap={2}>
      <div className="grid gap-2.5 sm:grid-cols-2">
        {SKILLS.map((s) => (
          <SkillCard key={s.name} name={s.name} description={s.description} tags={s.tags} />
        ))}
      </div>
      <span className="text-xs text-tertiary-label">
        不带 onToggle 时是纯展示卡片；描述超两行自动截断
      </span>
    </Stack>
  );
}
