# 狼人杀引擎 ECS 化 + 三库数据驱动（设计定稿）

> 单一事实源：DB 层 schema、引擎 ECS 契约、Server API、前端页面均以本文为准。
> 需求：技能可配置（内核+参数+时序）→ 角色=技能组合（可自由新增）→ 剧本=角色配比（可选开局）→ 全部 SQL 持久化；引擎全量 ECS；属性宏收敛内核样板。

## 1. 数据流总览

```
skills(SQL) ──组合──▶ roles(SQL) ──配比──▶ scripts(SQL) ──选择──▶ 开局
   kernel+params+时序      技能实例列表+阵营      角色数量+rules+intro      ScriptSetup
                                                                      │
                                              引擎 ECS：组件实例化 ◀──┘
                                              夜晚窗口 = 活人技能按 (phase,priority) 排序
```

## 2. 技能内核（kernel = 引擎内代码，唯一不可配置部分）

| kernel id        | 时序 phase | priority | 动作 payload           | 参数（params JSON）                          | 行为                                         |
| ---------------- | ---------- | -------- | ---------------------- | -------------------------------------------- | -------------------------------------------- |
| `wolf_chat`      | night      | 19       | `Text(Option<String>)` | —                                            | 狼队按座位序耳语，绕一圈进刀人               |
| `wolf_kill`      | night      | 20       | `Kill(u8)`             | —                                            | 狼队共享刀口（任意活狼可提交）               |
| `protect`        | night      | 10       | `Target(Option<u8>)`   | `{"noRepeat":true}`                          | 守护：禁连守同一人                           |
| `potions`        | night      | 30       | `Potions{save,poison}` | `{"saves":1,"poisons":1,"selfPoison":false}` | 解药救刀口/毒药毒人，同晚至多一瓶            |
| `inspect`        | night      | 40       | `Target(u8)`           | `{"selfTarget":false}`                       | 查验阵营，私有记录                           |
| `idol_bind`      | night      | 50       | `Target(u8)`           | `{"secret":true}`                            | 崇拜绑定：胜利条件跟随目标阵营，双方互不知情 |
| `duel`           | day        | 0        | `Target(Option<u8>)`   | —                                            | 白天翻牌决斗，一局一次，赌输以死谢罪         |
| `revenge_shot`   | passive    | 0        | `Target(Option<u8>)`   | `{"causes":["slain","exiled"]}`              | 死亡触发开枪窗口，级联                       |
| `reveal_survive` | passive    | 0        | —                      | —                                            | 被放逐翻牌免死、失去投票权                   |
| `none`           | —          | —        | —                      | —                                            | 无技能（平民）                               |

**夜晚结算系统（kill_resolve）**：每个刀口独立结算——被守 xor 被救=平安，被守且被救=奶穿死，都不=死；毒杀独立（被毒者不可触发 `revenge_shot`，按 `causes` 配置过滤）。
**胜负系统（outcome）**：狼全灭=好人胜；屠边（神职全灭或平民全灭）=狼胜；`idol_bind` 持有者的胜利跟随绑定目标的阵营。
**边界（诚实声明）**：技能配置 = 内核组合 + 参数 + 夜间时序；全新行为机制需新增内核代码。

## 3. 角色种子（roles ← skills）

| role id  | 阵营     | emoji | 技能                  |
| -------- | -------- | ----- | --------------------- |
| werewolf | werewolf | 🐺    | wolf_chat + wolf_kill |
| seer     | good     | 🔮    | inspect               |
| witch    | good     | 🧪    | potions               |
| guard    | good     | 🛡️    | protect               |
| hunter   | good     | 🏹    | revenge_shot          |
| idiot    | good     | 🤡    | reveal_survive        |
| knight   | good     | ⚔️    | duel                  |
| villager | good     | 👤    | none                  |

## 4. SQLite schema（rust/db，对齐现有 DbError 风格）

```sql
CREATE TABLE IF NOT EXISTS skills (
    id       TEXT PRIMARY KEY,          -- 'guard'
    name     TEXT NOT NULL,             -- '守卫'
    kernel   TEXT NOT NULL,             -- 'protect'（引擎内核 id）
    params   TEXT NOT NULL DEFAULT '{}',-- 内核参数 JSON（覆盖默认值）
    phase    TEXT NOT NULL,             -- night/day/passive
    priority INTEGER NOT NULL DEFAULT 0,
    targeting TEXT NOT NULL DEFAULT 'none', -- none/optional/required/group
    desc     TEXT NOT NULL DEFAULT '',
    builtin  INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY, name TEXT NOT NULL, camp TEXT NOT NULL,  -- werewolf/good
    emoji TEXT NOT NULL DEFAULT '👤', desc TEXT NOT NULL DEFAULT '',
    builtin INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS role_skills (
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    skill_id TEXT NOT NULL REFERENCES skills(id),
    ord INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (role_id, skill_id)
);
CREATE TABLE IF NOT EXISTS scripts (
    id TEXT PRIMARY KEY, name TEXT NOT NULL, total INTEGER NOT NULL,
    desc TEXT NOT NULL DEFAULT '', rules TEXT NOT NULL DEFAULT '[]',
    intro TEXT NOT NULL DEFAULT '[]', builtin INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
);
CREATE TABLE IF NOT EXISTS script_roles (
    script_id TEXT NOT NULL REFERENCES scripts(id) ON DELETE CASCADE,
    role_id TEXT NOT NULL REFERENCES roles(id),
    count INTEGER NOT NULL, ord INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (script_id, role_id)
);
```

- 种子迁移：首次启动（表空时）注入 §2 技能 + §3 角色 + 现有 3 剧本（newbie_six/nine_standard/all_stars，rules/intro 文案原样搬迁，builtin=1）。
- builtin 行禁改禁删（API 400）。

## 5. 引擎 ECS（rust/games/werewolf 重写）

### 组件（`ecs/comp.rs`，SoA 槽位 per seat）

```rust
pub struct SeatComp { pub role_id: String, pub camp: Camp, pub alive: bool, pub can_vote: bool }
pub struct SkillInst { pub skill_id: String, pub kernel: KernelId, pub params: KernelParams,
                       pub used: BTreeMap<String, bool> }  // 运行时状态（药水、决斗权、翻牌）
pub struct World {
    total: u8,
    seats: Vec<SeatComp>,
    skills: Vec<Vec<SkillInst>>,       // per seat
    checks: Vec<Vec<CheckRecord>>,     // 私有验人
    night: NightBoard,                 // kills/protects/saves/poisons 累计（天亮清空）
    last_protects: Vec<Option<PlayerId>>, // per protect 实例的连守记忆
    votes: Vec<Option<PlayerId>>, duel_offered: bool,
}
```

### 系统（`systems/`，`System` trait + 按 phase/priority 调度）

- `night_dispatch`：入夜生成窗口队列 `Vec<Window{seat, kernel}>`（活人技能按 phase/priority 排序），逐窗等动作
- `kill_resolve`（夜晚结算）、`day_speech`、`vote`、`duel_window`、`revenge_shot`、`outcome`、`idol_win`

### Action 协议（引擎-orchestr 内部，前端不直接发）

```rust
pub enum Action {
    Speech { text: String },
    Vote { target: Option<PlayerId> },
    Skill { kernel: KernelId, payload: Payload },
}
pub enum Payload { None, Target(Option<PlayerId>), Kill(PlayerId),
                   Potions { save: bool, poison: Option<PlayerId> }, Text(Option<String>) }
```

### 属性宏（rust/macros 新增 `#[skill_kernel]`）

```rust
#[skill_kernel(id = "protect", phase = "night", priority = 10,
               payload = "target_optional", params = "ProtectParams")]
impl Kernel for ProtectKernel { fn validate(...); fn commit(...); }
```

生成：`KernelDef` 常量、`KernelId` 匹配臂、params JSON 合并、payload 解码样板、action_contract 文案。中央注册表 `KERNELS: &[&KernelDef]` 每内核一行。

### 对外 API 保持

`GameEngine::new(setup, seed)`（`ScriptSetup { total, roles: Vec<RoleSpec{id, skills}> }`，原 BoardConfig 删除）、`pending() -> Option<(PlayerId, &'static str)>`（kind 字符串不变：狼人夜聊/白天发言/投票/夜晚行动…）、`apply() -> Vec<GameEvent>`、`snapshot(Viewer)`、`events()`、`phase().day()`、`players()`。GameEvent 枚举保留（NightStage 从枚举变 kernel 字符串）。测试 1240 行逐一迁移。

## 6. Server API（rust/server/routes/library + werewolf 改造）

- `GET/POST/PUT/DELETE /api/library/skills`（builtin 禁删改）
- `GET/POST/PUT/DELETE /api/library/roles`（`?detail=1` 附技能列表）
- `GET/POST/PUT/DELETE /api/library/scripts`（GET 附角色配比）
- `POST /api/game/start`：`board` 参数 → 从 DB 按 id 读剧本（`GameState::resolve_script`），未知/不可玩（无狼/无好人/未知内核）400
- utoipa schema 注册 + `export_openapi` 同步 → `pnpm codegen`

## 7. 前端（apps/web）

- `apis/werewolf.ts`：三库 CRUD；`boards.ts` 的 `toBoard` 改从 ScriptDto（含 roleLibrary）生成
- 新页面：剧本库（选剧本开局 + 新建：角色配比选择器）、角色库（新建角色：技能多选+阵营）、技能库（只读展示内核/参数/时序）
- RoomSetup 介绍步剧场化：片头卡（月夜背景+大标题）→ 板子字幕卡（半透明面板+阵营彩字，文案 100% 来自 script.rules/intro）→ CaptionBar 接 `/api/tts/speak` 逐段播报
- 素材：Seedream 生成月夜片头背景、烛光书房插画 → `apps/web/public/werewolf/`

## 8. 验收

`cargo test`（engine 行为全保留）+ `pnpm build/typecheck/lint/lint:rs` + `pnpm codegen` diff 干净 + 明暗主题肉眼过。
