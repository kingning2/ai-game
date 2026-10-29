# @agent-packages/shared-ui

组件库本体：Apple HIG 风格、令牌驱动的 React 组件，面向 AI Agent 控制台场景。

## 使用

```tsx
import { Button, Stack, WebSearch } from '@agent-packages/shared-ui';
import '@agent-packages/shared-ui/styles.css'; // 全局一次（Tailwind + 令牌 + reset）
```

暗色主题：`<html class="dark">` 即可，无需逐组件处理。

## 结构

- `src/<Name>/index.tsx` — 一个组件一个文件夹（PascalCase），同文件导出 `XxxProps` + 领域类型
- `src/layout/` — 布局组件特例（Stack/Divider/Container/NavBar/SectionHeader）
- `src/index.ts` — 唯一 barrel：所有公开导出（按分组注释排放）
- `styles/tokens.css` — 设计令牌（明暗两套），唯一允许出现色值的地方

## 组件清单

见 [COMPONENTS.md](./COMPONENTS.md)（新增/删除组件的规则也在此）。

## 约束

- 组件零业务逻辑、零内部数据状态：props 进、回调出，回调全可选
- 类名一律 `cn()`；动效只动 `transform/opacity`，一律 `ease-spring`
- peer 依赖 `react >= 18`，全仓 React 19 开发

## 命令

```bash
pnpm --filter @agent-packages/shared-ui build   # tsdown 构建
pnpm --filter @agent-packages/shared-ui dev     # watch 模式
```
