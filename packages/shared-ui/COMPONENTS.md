# Shared UI 组件规范

> `@agent-packages/shared-ui` 是本仓库唯一组件库：**Apple HIG 风格、令牌驱动、零业务逻辑**。
> 本文档是组件的**唯一权威说明**：每个组件是什么、怎么用、什么时候允许新增。
> 设计规则（主题/布局/滚动）见 `rules/web/app-design.md` 与 skill `app-design-system`。

## 1. 组件清单（当前全量，与 `src/index.ts` 导出一致）

### 布局（flex 优先）

| 组件      | 做什么                                                                                                   | 关键用法                                               |
| --------- | -------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| `Stack`   | flex 布局容器，替代手写 flex/div 堆叠。`direction` / `gap` / `align` / `justify`，别名 `VStack`/`HStack` | `<Stack gap={3} align="center">`                       |
| `Divider` | 发丝分割线（`separator` 令牌色）                                                                         | `<Divider />`                                          |
| `NavBar`  | iOS 风格导航栏：`leading` / `title` / `trailing` 三段，`sticky` 可选                                     | `<NavBar leading={btn} title="设置" trailing={btn} />` |

### 输入

| 组件     | 做什么                                                                                          | 关键用法                                                    |
| -------- | ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| `Button` | 按钮。variant：`solid`（主行动）/ `tinted` / `plain` / `gray` / `destructive`；size：`sm/md/lg` | `<Button variant="solid" size="sm">保存</Button>`           |
| `Input`  | 文本输入框（原生 input 封装，令牌化边框/焦点环）                                                | `<Input value={v} onChange={...} placeholder="..." />`      |
| `Slider` | 滑块（受控）：`value/min/max/step/onChange`                                                     | `<Slider min={256} max={8192} value={v} onChange={setV} />` |

### 展示

| 组件                | 做什么                                                                                                      | 关键用法                                                                                     |
| ------------------- | ----------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| `Avatar`            | 头像：`src` + 加载失败/无图时 `fallback` 文字；`xs/sm/md/lg`                                                | `<Avatar src={p.avatarSrc} fallback={p.name[0]} />`                                          |
| `Badge`             | 状态徽标。tone：`blue/tint/green/orange/red/gray`                                                           | `<Badge tone="green">进行中</Badge>`                                                         |
| `Card`              | 圆角浮起卡片容器（iOS inset grouped 规格）                                                                  | `<Card className="p-4">…</Card>`                                                             |
| `Spinner`           | 加载指示，`sm/md/lg`，可 `inline`                                                                           | `{loading && <Spinner size="sm" />}`                                                         |
| `SegmentedControl`  | iOS 分段控制（泛型受控）：`options: Segment edOption<T>[]` + `value` + `onChange`                           | `<SegmentedControl options={[{value:'openai',label:'OpenAI'}]} value={p} onChange={setP} />` |
| `List` / `ListItem` | iOS 分组列表。`ListItem`：`media` / `title` / `subtitle` / `trailing` / `onClick`，透传 `...HTMLAttributes` | `<List><ListItem media={<Avatar/>} title="豆包" trailing={<Badge/>} onClick={...}/></List>`  |

### AI 场景

| 组件            | 做什么                                                                                                                         | 关键用法                                                                                                                      |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| `WerewolfStage` | 狼人杀对局舞台：座位环绕、聚光灯、阶段氛围（夜/昼）、`fullBleed` 全屏；children 为舞台中央内容。配套类型 `WerewolfPlayer`      | `<WerewolfStage players={ps} phase="night" phaseLabel="第 1 夜" backgroundUrl="/bg.jpg" fullBleed>{children}</WerewolfStage>` |
| `SettingsPanel` | AI 供应商设置表单（与 Rust `/api/agent/config` 契约对齐）：provider 分段切换 + baseUrl/model/apiKey/maxTokens（内建 `Slider`） | `<SettingsPanel value={v} draft={d} onDraftChange={...} onSave={...} />`                                                      |

## 2. 使用规则

1. **先库里找，再动手写**：页面需要的 UI 优先复用上表组件；shared-ui 没有**且确属通用**才允许新增（见 §3）。
2. **页面禁止绕过组件造轮子**：iOS 分组行必须用 `List + ListItem`（不许手搓 divide-y 行卡）；分段切换必须用 `SegmentedControl`（不许手写 toggle 按钮组）。
3. **零业务逻辑、零内部数据状态**：数据组件一律受控（`value + onChange`），回调可选。业务/请求逻辑放页面或 server。
4. **颜色只走令牌**：组件里只许用语义类（`text-label`、`bg-fill`…），禁止 hex/`bg-black`。app 侧语义（arena）不进 shared-ui。
5. **`cn()` 拼条件类**，禁止模板字符串手拼。
6. **动效**只动 `transform/opacity`，`ease-spring`，入场 `scale(0.9+)+opacity`。

## 3. 新增通用组件前，必须想清楚（checklist）

逐条自问，任何一条答"否"就不许加：

- [ ] **第二使用者是谁？** 只有一个页面的东西写在页面里；能说出第二个明确使用场景（或 3 个潜在场景）才进库。
- [ ] **是不是已有组件的变体？** 能用 prop（variant/size/tone）扩展旧组件解决的，不新建。
- [ ] **是不是业务件？** 含游戏规则/请求/状态机的不进 shared-ui（业务件放 `apps/web/src/<domain>/`）。
- [ ] **契约是否受控？** props 全受控、无内部数据状态、无副作用。
- [ ] **令牌齐全？** 没有新硬编码色值；需要新语义色先进 `tokens.css`。
- [ ] **文档齐了吗？** `src/<Name>/index.tsx` 写 TSDoc 头注释（做什么 + props 说明），这是唯一的组件文档。
- [ ] **验收**：`pnpm build`、`pnpm typecheck` 全绿。

## 4. 删除组件的规则

- 组件被 `apps/web` 之外无引用、且 web 应用不再使用 → 删除（组件目录 + `index.ts` 导出）。
- **被其他保留组件 import 的基础件不能删**（如 `Slider` 被 `SettingsPanel` 使用）。
- 删除后必须：shared-ui build + web typecheck 两绿。

## 5. 目录结构

```
packages/shared-ui/
├── src/
│   ├── layout/          # Stack / Divider / NavBar
│   ├── <Component>/index.tsx   # 一组件一目录，默认导出 + 命名导出 Props 类型
│   ├── cn.ts            # 类名拼接工具
│   └── index.ts         # barrel 导出（唯一对外出口）
├── styles/tokens.css    # ①层令牌：iOS 系统色/字体/动效（明暗两套）
└── styles.css           # 令牌 + @theme + 动画 keyframes
```
