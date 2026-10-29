# @agent-packages/game

狼人杀游戏前端基础类库：枚举、回放节奏控制、replay 状态机。**框架无关、传输层无关**（不依赖 React，不关心消息来自 WebSocket 还是本地 mock）。

## 内容

- `src/game.ts` — 游戏领域类型与枚举
- `src/playback.ts` — 回放节奏控制与状态机

## 约束

- 不 import React；UI 层在 `apps/web/src/werewolf/`
- 状态推进由调用方驱动，库内不做时序假设

## 命令

```bash
pnpm --filter @agent-packages/game build   # tsdown 构建
pnpm --filter @agent-packages/game dev     # watch 模式
```
