import { AgentHandoff } from '@agent-packages/shared-ui';

/** 交接回执：不是裸箭头 —— 带走什么 / 刻意留下什么 / 为什么交接，全部可审计 */
export default function AgentHandoffBasic() {
  return (
    <AgentHandoff
      from="Researcher"
      to="Writer"
      reason="起草纪要需要写作专家，不是再搜一轮"
      carried={['续约条款摘录 (p.4)', 'SLA 违约次数：2', '语气：正式']}
      withheld={['原始搜索日志', '无关竞品笔记']}
      time="09:40:41"
    />
  );
}
