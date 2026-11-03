# 狼人杀引擎 Bevy ECS 改造 — 审计报告与迁移计划（阶段 1 交付）

> 按 spec §3 输出：现状架构图、迁移映射表、保留/迁移/排除清单、分阶段计划。
> 上一份 `docs/werewolf-ecs.md` 的「手写 SoA World」引擎设计由本文 **取代**（其 §4 SQLite 三库 schema、§6 API、§7 前端章节仍然有效）。
> 审计期间未修改任何代码。

## 1. 现状架构图

```
React 前端 (packages/game)                      Agent (rust/agent)
  ClientGame：seq 去重 / playhead / 快照订阅      LLM 决策（reqwest 异步）
        ▲  HTTP + SSE (seq)                        ▲ 工具调用（强制）
        │                                          │
┌───────┴───────────── server (axum) ─────────────┴──────────┐
│ routes/werewolf  start/pause/resume/end/status/事件流(SSE)  │
│   事件 seq = 引擎日志下标；按 (seen, len) 增量推送           │
│ orchestr/autoplay  决策环：锁内收上下文 → 锁外 LLM → 锁内 apply │
│ orchestr/memory    recall(engine, player)：事件流→玩家记忆    │
│ orchestr/mvp       赛后 MVP 评定（读事件流）                  │
└───────────────────────────┬────────────────────────────────┘
                            │ Arc<AppState>.game
┌───────────────── state/game_state.rs ──────────────────────┐
│ 生命周期：Idle→Running⇄Paused→Ended（409 语义）             │
│ 多槽位：game_id → { lifecycle, engine: Option<GameEngine> } │
│ 访问面：with_engine / with_engine_mut（std Mutex 闭包，禁 await）│
└───────────────────────────┬────────────────────────────────┘
                            │ 引擎 API（唯一被外部消费的面）
┌──────────────────── games/werewolf ────────────────────────┐
│ GameEngine：手写状态机（非 ECS）                             │
│   状态：total/players(Vec<PlayerState>)/phase/events/potions │
│         /night/votes/duel_offered/wolf_chat/last_protected  │
│   推进：apply(actor, Action) 校验→推进→返回增量事件           │
│   事件：events: Vec<GameEvent>（上帝日志，seq=下标，唯一事实源）│
│   视图：snapshot.rs 视角裁剪 / summary.rs 对局总览            │
│   纯逻辑：role/phase/action/board/error（零 IO，可整体保留）  │
└────────────────────────────────────────────────────────────┘
```

关键事实：

- **事件日志是唯一事实源**：memory/mvp/SSE/前端回放全部从 `Vec<GameEvent>` 派生；snapshot/summary 是投影 DTO。
- **Action 只在 server 内部产生**（autoplay LLM 工具调用解析）；前端从不直接发动作 → 改 Action 爆破半径 = games/werewolf + orchestr。
- **锁纪律**：LLM 调用在锁外（spec §7 现状已满足）；`with_engine_mut` 闭包内禁 `.await`。
- **client 契约**：seq 去重 + playhead 逐条演出，seq = 日志下标，必须保持单调连续。

## 2. 迁移映射表

| 现有代码                                                             | 去向                            | 处置                                                                                                                           |
| -------------------------------------------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `role.rs`（Role/Camp/枚举+阵营）                                     | **game-domain**                 | 原样保留（纯类型，规则不可依赖 ECS）                                                                                           |
| `phase.rs`（Phase/NightStage/Outcome/DeathCause/Viewer/CheckRecord） | **game-domain**                 | 保留 serde wire 形状；Phase 运行时形态改为 ECS Resource，wire 仍出旧形状                                                       |
| `action.rs`（Action：玩家意图）                                      | **game-domain**（=GameAction）  | 保留；「意图 vs 事实」语义已正确                                                                                               |
| `board.rs`（BoardConfig+3 预设）                                     | **game-domain**                 | 保留，后续 SQL 三库阶段扩为 ScriptSetup                                                                                        |
| `error.rs`                                                           | **game-domain**                 | 原样保留                                                                                                                       |
| `summary.rs`（GameSummary，从日志派生）                              | **game-domain**                 | 原样保留（纯函数）                                                                                                             |
| `event.rs`（GameEvent：业务事实）                                    | **game-events**                 | 原样保留 + seq 显式化（日志下标即 seq，行为不变）；**不是** bevy Message                                                       |
| `snapshot.rs`（视角裁剪 + DTO）                                      | **game-events**                 | 保留为显式 DTO Projection：输入 = game-ecs 构建的只读投影模型，输出 wire DTO；不序列化 World                                   |
| `player.rs`（PlayerState 大结构体）                                  | **game-ecs 组件**               | **拆分**：`Seat{alive,can_vote}`、`RoleOf{role}`、`PrivateChecks`、`PotionState`、`DuelUsed`、`IdiotRevealed`（单一大组件 ❌） |
| `engine.rs`（GameEngine 状态机）                                     | **game-ecs systems**            | 按 §5 拆 System；`GameEngine` 外壳暂留为门面（见 §4 兼容策略）                                                                 |
| `Rng`（splitmix+Fisher-Yates）                                       | **game-ecs Resource**           | 原样搬移，种子语义（同 seed 同发牌）不变                                                                                       |
| `state/game_state.rs`（槽位+生命周期）                               | **game-runtime**（GameManager） | 生命周期语义/409/多槽位全保留；槽位值从 `Option<GameEngine>` 换成实例运行时句柄                                                |
| `orchestr/autoplay`                                                  | **server 适配层**（不变位置）   | 保留决策环形状：锁内收集→锁外 LLM→锁内 apply；apply 走门面                                                                     |
| `orchestr/memory`、`mvp`                                             | server（不变）                  | 零改动：只读事件日志                                                                                                           |
| `db`（games/事件持久化、record）                                     | server 侧（不变）               | 零改动                                                                                                                         |
| SSE / seq / 前端 packages/game                                       | 不变                            | 契约冻结，测试守护                                                                                                             |

**不入 ECS（明确排除）**：GameEvent 日志（保持普通 Vec，是回放/seq/持久化的事实源）、snapshot DTO、GameSummary、agent、db、tts、SSE 路由。

## 3. 目标 crate 结构与依赖边界

```
game-domain（新）  纯类型+规则：role/phase(类型)/action/summary/error/board
game-events（新）  GameEvent/日志/快照 DTO 投影/回放辅助        ← 依赖 game-domain
game-ecs（新）     bevy_ecs：components/resources/systems/schedules/plugins
                                                     ← 依赖 bevy_ecs + game-domain + game-events
game-runtime（新）  实例生命周期（GameManager=原 game_state）、调度驱动 ← 依赖以上全部
state / server / agent / db / tts / macros  依赖面不变（经由门面平稳过渡）
```

- `bevy_ecs` 精确 pin 进根 `[workspace.dependencies]`（加 crate 前过 rust-cargo-workspace-designer 检查）。
- 禁止循环依赖：server/agent 永不直接触碰 `bevy_ecs::World`。

## 4. 兼容策略（防一次性重写）

1. **门面壳**：`games/werewolf` 的公开 API（`GameEngine::new(board,seed)/apply/pending/snapshot/events/phase/players`、类型 re-export）**逐字保留**；内部实现逐步从手写状态机切换到 game-runtime 驱动的 ECS World。server/state 编译面在整个迁移期不动。
2. **黄金测试**：现有 `engine_tests.rs`（1240 行）+ autoplay_tests 全部保留作为行为黄金基线；迁移前后同一动作序列必须产出逐字节相同的 `Vec<GameEvent>`（seq 对齐）。
3. **apply 语义**：门面 `apply()` = 校验 → 驱动 Schedule 到下一个等待点（window）→ 返回增量事件；系统内零阻塞、零 LLM，spec §7 天然满足（决策环本就在锁外）。

## 5. System / Schedule 设计（目标态）

```
Schedule:Setup      PlayerSpawnSystem（发牌洗牌，Rng 资源）
Schedule:Night      守卫→狼聊→狼刀→女巫→预言家（SystemSet 顺序固定；当前窗口=Resource）
Schedule:Resolution KillResolveSystem（守护/解药/奶穿/毒杀独立结算）→ PlayerDeathSystem → DeathTriggerSystem(猎人/胜负)
Schedule:Day        DaySpeechSystem → DuelOfferSystem
Schedule:Voting     VotingSystem（收票）→ VotingResolutionSystem（白痴翻牌/放逐/猎人窗口）
Schedule:CheckWin   WinConditionSystem（屠边+偶像绑定）
Schedule:Cleanup    夜间累计清空 / 阶段推进（PhaseTransitionSystem 独占改阶段权）
```

- 系统**不得**各自改 Phase：阶段推进只归 `PhaseTransitionSystem`（spec §5）。
- 当前等待窗口 = Resource（seat + kind 字符串），`pending()` 从它派生。
- 每局独立 `World + Schedule`（GameManager 槽位），Tokio 管生命周期；实例间并行、实例内 apply 串行（沿用现有 Mutex 槽位模型平滑过渡，消息通道化作为 runtime 阶段收尾项）。

## 6. 分阶段实施顺序（= spec §12，每步跑黄金测试）

1. ✅ 审计 + 本文档
2. 建 game-domain（搬纯类型）→ 建 game-events（搬事件/快照）→ games/werewolf 改引用，全绿
3. 建 game-ecs（bevy_ecs + 组件/资源）→ 迁 Player/Phase（ systems 尚空，门面仍走旧逻辑）
4. 迁投票 System → 迁夜间 System → 迁胜负 System（每步黄金测试）
5. Event Log/Replay 接线（日志资源化，seq 契约测试）
6. game-runtime：GameManager 替换 game_state 槽位内部（生命周期 API 不变）
7. Agent 适配层确认（预计零改动）→ server/SSE 接入 → 全量兼容测试
8. 收尾：删除 engine.rs 旧实现 + 门面瘦身；SQL 三库阶段按 `docs/werewolf-ecs.md` §4/§6/§7 继续

## 7. 宏策略（spec §10）

- 阶段 1–7 **不写**自定义 ECS 宏；Bevy 原生 derive（Component/Resource/Bundle/SystemParam）+ 普通函数注册。
- 既有 `#[game_event]`（rust/macros，生成 day_of 等派生）继续沿用；`#[skill_kernel]` 属性宏推迟到 SQL 三库阶段（内核注册处真出现重复时再上）。
- 用户此前要求的「属性宏减重复」落在三库阶段的内核注册宏，与 spec §10「发现实际重复才做」一致。

## 8. 风险与未决

- **bevy_ecs 版本**：需 pin 精确 minor 并跑 clippy 37 条 deny 清单；bevy_ecs 部分lint 与 workspace 清单可能冲突（届时在 workspace.lints 层面解决，不开局部 allow）。
- **确定性**：系统执行顺序必须全显式（SystemSet 链），禁止 bevy 隐式歧义排序，否则同 seed 事件流不可复现。
- **黄金测试逐字节对比**：GameEvent 含 serde 输出，直接 diff 序列化 JSON 最稳。
- **Tauri 壳**（apps/desktop/src-tauri 在 workspace 成员里）依赖 state 面，门面策略保证其不动。
- 未决：spec §6「消息通道接收指令」与现有 Mutex 槽位并存策略 —— 先 Mutex 平滑迁移，通道化放 runtime 收尾单独评审。
