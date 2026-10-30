# Rust 游戏引擎架构规则（rust/ 目录，强制）

> 适用范围：`rust/` 下所有代码改动。权威架构说明见 `docs/werewolf-ecs.md`（spec）与 `docs/werewolf-ecs-migration.md`（迁移审计）；两者与本文件冲突时以本文件为准。

## Crate 拓扑与依赖红线

```
game-domain（纯领域：角色/板子/阶段/剧本/RNG/PlayerState，零 IO、零 bevy）
  ↑                    ↑
game-events（GameEvent 业务事实流）   game-ecs（bevy_ecs 0.20.1：组件/资源/System）
  ↑                    ↑
  werewolf（门面：GameEngine 转发壳 + 快照视角裁剪）← game-runtime（GameManager 生命周期）
```

依赖只允许自下而上单向：`domain ← events ← ecs`，`domain ← ecs`，`domain+events+ecs ← runtime/werewolf`。禁止任何反向依赖或跨层跳依赖（如 server → ecs）。

1. **领域规则只能写在 `game-domain`**：角色能力、板子校验、阶段定义、胜负条件涉及的纯类型与判定。禁止在 server / ecs / runtime 里重复定义领域类型。
2. **业务事实只能是 `game-events::GameEvent`**：新增游戏事实必须加 GameEvent 变体（serde 带文档注释），禁止另开日志/通知通道。事件 seq = 日志下标，单调连续，是 SSE 去重与前端回放契约，禁止破坏。
3. **游戏行为只能写在 `game-ecs` 的 System/模块函数里**（每阶段一段：`night.rs` / `day.rs` / `flow.rs`）。禁止把游戏逻辑塞回 `werewolf`（它现在只是 `game-runtime::GameManager` 的逐字转发壳 + snapshot 视角层）或塞进 server 路由。
4. **对外 API 面只能经 `werewolf::GameEngine` 门面**：server / agent 调 `new / apply / pending / phase / events / settled / players`。给 runtime/ecs 加能力时，先在门面补转发再改调用方，禁止 server 直接依赖 `game-ecs` / `game-runtime` 类型。
5. **ECS 边界纪律**：业务身份用 `game_domain::phase::PlayerId`（组件 `PlayerMark`），Bevy `Entity` 不出 `game-ecs`；事件日志是普通 Resource（`EventLog`），不是 ECS Message/Observer；每个玩家字段一组件（PlayerState 拆分见 `game-ecs/src/components/player.rs`），禁止把大结构体塞成单组件。

## 行为等价守护（改引擎必读）

6. **黄金对拍测试不许弱化**：`game-ecs/tests/golden_full.rs`（随机动作序列对拍 + 白痴投票权回归）与 `golden_setup.rs`（开局日志全量比对）是行为基准。改发牌/结算/阶段流转前先跑；改完必须全绿，任何 diff 都要能解释成「有意的语义变更」并同步更新 doc 注释。
7. **`seed` 确定性契约**：同 `seed` ⇒ 同发牌 ⇒ 同事件序列（`game_domain::Rng` xorshift+splitmix64）。禁止在游戏路径引入真随机、HashMap 迭代序或浮点。

## 新增内容的最短路径

- 新角色/新板子：只动 `game-domain`（role/profile/board），ecs 阶段函数按 role 分支的补分支 + 对拍测试补板子。
- 新事件：`game-events` 加变体 → 发出方（ecs 对应阶段）push → 前端 `packages/shared` types 走 `pnpm codegen` 同步。
- 新 API：先 `game-runtime`/门面加方法 → server 路由调门面 → openapi 导出（`pnpm export_openapi`）。

## 交付验收（rust/ 改动）

`cargo build --workspace && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings` 全绿，加 `/tmp/manifest_lint . --strict && /tmp/dep_audit . --strict`（AGENTS.md Rust 节）。涉及 openapi 契约的再跑 `pnpm build && pnpm typecheck`。

**测试组织**：单元测试一律放独立的 `*_tests.rs` 文件（源文件底部 `#[cfg(test)] #[path = "xxx_tests.rs"] mod tests;` 声明，测试文件 `use super::*;`），禁止内联 `mod tests`——对齐 `tts/src/doubao_tests.rs`、`games/werewolf/src/*_tests.rs` 既有模式；集成/黄金测试放 `tests/` 目录。
