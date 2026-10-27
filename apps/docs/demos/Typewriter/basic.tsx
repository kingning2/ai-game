import { Typewriter } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const TEXT = '你好！我是你的 AI 助手。这是一段用打字机效果逐字输出的回复，模拟流式生成的阅读体验。';

export default function TypewriterBasic() {
  return (
    <Stack gap={4}>
      <Typewriter text={TEXT} />
    </Stack>
  );
}
