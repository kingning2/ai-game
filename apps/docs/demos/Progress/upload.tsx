import { Progress } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const files = [
  { name: '产品需求文档.pdf', value: 100 },
  { name: '设计稿-首页.fig', value: 62 },
  { name: '数据导出.csv', value: 8 },
];

/** 组合用法：文件名 + 百分比 + 进度条组成上传清单 */
export default function ProgressUpload() {
  return (
    <Stack gap={4} className="w-full max-w-100">
      {files.map((f) => (
        <Stack key={f.name} gap={1}>
          <div className="flex items-center justify-between text-xs">
            <span className="text-secondary-label">{f.name}</span>
            <span className="font-mono tabular-nums text-tertiary-label">{f.value}%</span>
          </div>
          <Progress value={f.value} />
        </Stack>
      ))}
    </Stack>
  );
}
