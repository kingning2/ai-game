import { WerewolfStage, type WerewolfPlayer } from '@agent-packages/shared-ui';

/** 12 名 Agent 的上帝视角对局（4 狼 + 4 神 + 4 民），第 2 天白天 */
const PLAYERS: WerewolfPlayer[] = [
  { seat: 1, name: '豆包', roleLabel: '狼人', roleTone: 'wolf' },
  { seat: 2, name: '元宝', roleLabel: '女巫' },
  { seat: 3, name: '智谱清言', roleLabel: '平民' },
  { seat: 4, name: 'DeepSeek V3', roleLabel: '预言家' },
  { seat: 5, name: '通义千问', roleLabel: '狼人', roleTone: 'wolf', alive: false },
  { seat: 6, name: 'Gemini 2.5', roleLabel: '平民', alive: false },
  { seat: 7, name: 'Kimi', roleLabel: '平民' },
  { seat: 8, name: 'GPT-4o', roleLabel: '守卫' },
  { seat: 9, name: '文心一言', roleLabel: '平民' },
  { seat: 10, name: 'DeepSeek R1', roleLabel: '狼人', roleTone: 'wolf' },
  { seat: 11, name: 'GPT-5', roleLabel: '狼人', roleTone: 'wolf' },
  { seat: 12, name: 'Grok 4', roleLabel: '猎人' },
];

/** 基础用法：左右座位列 + 中央发言卡 + 底部大字幕 */
export default function WerewolfStageBasic() {
  return (
    <WerewolfStage title="AI 狼人杀" players={PLAYERS} caption="但最后好多一枪带走了个狼">
      <div className="mx-auto w-full max-w-xl rounded-xl bg-background p-4 shadow-popover">
        <div className="flex items-center gap-2.5">
          <span className="flex size-8 shrink-0 items-center justify-center rounded-full bg-tint/10 text-sm font-medium text-tint">
            智
          </span>
          <span className="text-sm font-semibold text-label">智谱清言</span>
          <span className="flex items-center gap-1 text-xs text-blue">
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-blue" />
            发言中
          </span>
        </div>
        <p className="mt-3 border-l-2 border-blue/40 pl-3 text-sm leading-relaxed text-label">
          来来来，我们复盘一遍——1 号开场站边没问题，但人家跳预之后被回头刀。6
          号被救回来那轮突然跳守卫，9 号才是真平民。狼只剩一个：5
          号通义千问，从头到尾都在给狼队当挡箭牌。今天这票不用犹豫了。
        </p>
      </div>
    </WerewolfStage>
  );
}
