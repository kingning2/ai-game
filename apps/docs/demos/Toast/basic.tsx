import { Button, ToastProvider, useToast } from '@agent-packages/shared-ui';

function Trigger() {
  const { toast } = useToast();
  return (
    <div className="flex flex-wrap gap-2">
      <Button variant="gray" size="sm" onClick={() => toast({ title: '已保存草稿', kind: 'info' })}>
        info
      </Button>
      <Button
        variant="gray"
        size="sm"
        onClick={() =>
          toast({
            title: '账单导出完成',
            description: 'ark-billing-2025-06.csv · 12.4 KB',
            kind: 'success',
          })
        }
      >
        success
      </Button>
      <Button
        variant="gray"
        size="sm"
        onClick={() =>
          toast({ title: '部署失败', description: 'Endpoint ep-2024… 配额不足', kind: 'error' })
        }
      >
        error
      </Button>
    </div>
  );
}

/** ToastProvider 包裹应用根，内部用 useToast() 触发；横幅从顶部滑入、点击即关、3.2s 自动消失 */
export default function ToastBasic() {
  return (
    <ToastProvider>
      <Trigger />
    </ToastProvider>
  );
}
