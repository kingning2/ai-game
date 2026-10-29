# @agent-packages/shared

纯逻辑包（无 React）：领域类型、判别函数、请求层。

## 内容

- `src/is.ts` — `is*` 判别函数（错误分流等）
- `src/request/` — 请求层：`ApiError`、`ApiEnvelope<T>`、fetch 封装

## 使用

```ts
import { isApiError, ApiError } from '@agent-packages/shared';
```

## 约束

- **禁止 import React**（纯逻辑边界，见 `rules/common/project-structure.md`）
- 错误分流由 `ApiError` 与 `is*` 判别方法承担，调用方不写 `useUnknownInCatchVariables`
- 构建用 `tsdown`，产物 `dist/index.mjs` + `dist/index.d.mts`

## 命令

```bash
pnpm --filter @agent-packages/shared build   # tsdown 构建
pnpm --filter @agent-packages/shared dev     # watch 模式
```
