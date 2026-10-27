import { Markdown } from '@agent-packages/shared-ui';

const DOC = `### 周报汇总

| 模块 | 进度 | 负责人 |
| --- | --- | --- |
| 导航栏 | 已完成 | 林一 |
| 数据表格 | 进行中 | 陈二 |
| 灯箱 | 排期中 | 张三 |

- [x] 侧边栏路由修正
- [x] demo 登记脚本
- [ ] 全站构建巡检

~~已废弃的旧版接口~~ 不再维护。`;

export default function MarkdownGfm() {
  return (
    <div className="rounded-xl border border-separator bg-background p-4">
      <Markdown text={DOC} />
    </div>
  );
}
