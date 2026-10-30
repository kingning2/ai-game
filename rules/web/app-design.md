# 应用设计规则（apps/web 全页面适用）

> 本文件是 `apps/web` 页面级 UI 的**强制规范**。组件库通用规范见 `rules/web/ui-design.md` 与
> `rules/web/component-guide.md`；本文只管**应用层**：主题、页面框架、滚动、双气质。
> 配套 skill：`.agents/skills/app-design-system/SKILL.md`（含好/坏示例）。

## 1. 三层令牌体系（颜色只从令牌来）

| 层           | 文件                                        | 内容                                                                         | 谁能改                                            |
| ------------ | ------------------------------------------- | ---------------------------------------------------------------------------- | ------------------------------------------------- |
| ① 通用层     | `packages/shared-ui/styles/tokens.css`      | iOS 系统色、字体栈、动效曲线                                                 | 组件库级改动才动                                  |
| ② 应用语义层 | `apps/web/src/index.css` 的 `@theme inline` | `arena` 竞技场语义色（沉浸暗色游戏侧）                                       | 新语义色在这里加                                  |
| ③ 消费层     | 各页面/组件                                 | **只许用语义类**（`bg-arena`、`text-arena-muted`、`bg-grouped-background`…） | 禁止 hex/rgb 字面量、禁止 `bg-black`/`text-white` |

语义色当前清单（②层）：

| 令牌                                       | 用途                                       |
| ------------------------------------------ | ------------------------------------------ |
| `--color-arena`                            | 舞台底色（近黑夜色，恒定不随明暗主题翻转） |
| `--color-arena-raised`                     | 浮起面板/卡片底                            |
| `--color-arena-line`                       | 发丝描边/分隔                              |
| `--color-arena-text` / `-muted` / `-faint` | 主/次/弱文字                               |
| `--color-arena-gold`                       | 主强调（狼人杀金）                         |
| `--color-arena-glow`                       | 氛围光/链接（蓝）                          |
| `--color-arena-danger` / `-ok`             | 危险/成功                                  |

**加新颜色的流程**：先问"这是不是新语义？"→ 是则加到②层并命名 → 页面引用；不是 → 复用现有语义或①层系统色。**永远跳过直接写色值**。

## 2. 双气质分区

| 区域                 | 页面                                     | 主题                                           |
| -------------------- | ---------------------------------------- | ---------------------------------------------- |
| 竞技场（游戏沉浸侧） | 游戏大厅、狼人杀大厅、创建向导、对局直播 | 恒定暗色：`bg-arena` 系令牌，金色点缀，戏剧性  |
| 控制台（设置侧）     | AI 设置、选手模型                        | 跟随明暗主题：`grouped-background` 系 iOS 分组 |

同一页面内不得混用两套气质；游戏侧不出现 iOS 分组卡，设置侧不出现黑金氛围。

## 3. 页面框架铁律（app frame，零 body 滚动）

- `html/body/#root` 已在 `index.css` 全局锁死（`height:100%; overflow:hidden; overscroll-behavior:none`）——**任何页面不得再出现页面级滚动**。
- 每页固定结构：

```tsx
<div className="flex h-dvh flex-col overflow-hidden bg-arena">
  {/* 固定区：NavBar / 顶栏（不随内容滚） */}
  <div className="min-h-0 flex-1 overflow-y-auto">{/* 内容区：唯一滚动方 */}</div>
</div>
```

- 长内容（房间列表、排行榜、日志、表单）滚在**自己的内容区**里；
- 全屏舞台页（对局直播）可以没有滚动区——所有面板内部消化溢出（日志列 `min-h-0` + 内部 `overflow-y-auto`）；
- 滚动条样式已全局统一（6px 圆角半透明，hover 加深），不要在页面里再自定义。

## 4. 间距与节奏

- 竞技场页内容列：`mx-auto max-w-5xl px-6 py-10`（向导页 `max-w-3xl`）；区块间距 `gap-8`（Hub 首页 `gap-12`）。
- 设置页内容列：`mx-auto max-w-md px-6`，分组卡间距 `gap-3`，节标题（`text-xs font-medium uppercase tracking-wide text-secondary-label`）。

## 5. 数据展示区

- **排行榜 = `grid grid-cols-2 gap-3` 两列卡**（头像 + 名次徽章 + 胜/率/MVP），不用单列长表。
- 列表型数据优先复用 shared-ui `List + ListItem`；卡片型（多字段摘要）用 `Card` 或 grid 卡组。

## 6. 动效（沿用 ui-design.md，重申）

- 只动 `transform/opacity`；`ease-spring`；入场 `scale(0.9~0.95) + opacity`，禁 `scale(0)`。

## 7. 验收清单（每次动页面过一遍）

1. `pnpm --filter @agent-packages/web typecheck && pnpm --filter @agent-packages/web build` 全绿
2. `grep -nE "bg-black|text-white[^/]|#[0-9a-fA-F]{3,6}" apps/web/src/**/*.tsx` 无新增字面量（半透明 `white/xx` 用于阴影高光层可豁免，但优先 `arena-raised/line`）
3. 浏览器逐页检查：body 零滚动（`document.body.scrollHeight === clientHeight`），滚动条只出现在内容区
4. 明暗两套主题各过一遍（竞技场恒暗、控制台跟随）
