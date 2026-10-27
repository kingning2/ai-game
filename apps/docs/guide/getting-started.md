# 快速上手

## 安装

组件库随 monorepo 发布，业务应用以 workspace 依赖接入：

```bash
pnpm add @agent-packages/shared-ui
```

## 引入

### 1. 组件

```tsx
import { Button, Input } from '@agent-packages/shared-ui';
```

### 2. 样式（必须，一次即可）

```tsx
import '@agent-packages/shared-ui/styles.css';
```

样式入口包含三部分：

1. **Tailwind v4 编译层**：`@import "tailwindcss"` + 内容扫描
2. **设计令牌**：`tokens.css` 里的 `--tint`、`--fill`、`--shadow-card` 等变量（明暗两套）
3. **基础 reset**：极薄的一层，不污染业务样式

### 3. 深色模式

根元素挂 `.dark` 类即可整体切换（令牌层完成变量翻转）：

```html
<html class="dark"></html>
```

## 最小示例

```tsx
import { Button, Input, Stack } from '@agent-packages/shared-ui';
import '@agent-packages/shared-ui/styles.css';

export default function App() {
  return (
    <Stack spacing={12} className="p-8">
      <Input placeholder="你叫什么名字" />
      <Button>提交</Button>
    </Stack>
  );
}
```

## 在 VitePress / 任意站点中渲染 React demo

本站所有示例都是真实 React 实例：`.vitepress/components/DemoPreview.vue` 通过 `createRoot` 把 `demos/index.ts` 登记的组件挂载到 Vue 页面中，源码用 `<<<` 语法从同一文件嵌入——**示例即真源，永不失真**。

## 设计理念

见 [设计理念（令牌驱动）](/guide/design)。
