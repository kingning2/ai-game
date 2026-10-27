import { Card } from '@agent-packages/shared-ui';

export default function CardBasic() {
  return (
    <Card className="max-w-md">
      <h3 className="text-sm font-semibold text-label">付款方式</h3>
      <p className="mt-1 text-xs text-secondary-label">
        更新用于 App Store 与订阅的付款信息，所有交易均通过安全通道处理。
      </p>
    </Card>
  );
}
