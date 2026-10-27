import { PromptSuggestions } from '@agent-packages/shared-ui';

/** 胶囊式（默认）与列表式（stacked，引导页）两种形态，stagger 入场 */
export default function PromptSuggestionsBasic() {
  return (
    <div className="flex flex-col gap-5">
      <PromptSuggestions
        title="试试这些"
        prompts={[
          '查一下本月账单',
          '用 Seedream 生成一张海报',
          '精调任务 mcj-1a2b 进展如何',
          '把这段话翻译成英文',
        ]}
        onSelect={() => {}}
      />
      <PromptSuggestions
        stacked
        title="列表式（引导页）"
        prompts={['接入 Doubao 到我的应用', '把 ARK 接入到现有服务']}
        onSelect={() => {}}
      />
    </div>
  );
}
