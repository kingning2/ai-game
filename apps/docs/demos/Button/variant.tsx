import { Button } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

export default function ButtonVariant() {
  return (
    <Stack spacing={3}>
      <Stack direction="row" spacing={3} wrap>
        <Button>保存修改</Button>
        <Button variant="tinted">添加标签</Button>
        <Button variant="plain">查看详情</Button>
      </Stack>
      <p className="m-0 text-xs text-tertiary-label">
        solid 用于页面唯一主操作；tinted / plain 用于次级；destructive 只用于不可逆危险操作。
      </p>
    </Stack>
  );
}
