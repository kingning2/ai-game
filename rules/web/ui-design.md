# UI 设计规范（Apple HIG · 令牌驱动）

> 本文件覆盖 `rules/common/` 中与 UI 相关的约定。设计理念版见组件源码 TSDoc 与 `apps/web` 演示场。

## 三层令牌（架构基石）

```
styles/tokens.css        CSS 变量，明暗两套（唯一允许出现色值的文件）
   ↓ @theme
Tailwind v4 工具类       bg-tint / text-label / shadow-card …
   ↓
组件                     只消费工具类，零硬编码
```

- 新增令牌：先在 `tokens.css` 定义亮/暗两套 → `@theme` 暴露 → 组件用工具类。三步缺一不可。
- 暗色模式：根元素挂 `.dark`，令牌层自动翻转；组件内**禁止**写 `dark:` 前缀分支。
- 机械强制：`.pi/extensions/repo-guard.ts` 拦截 edit/write 中的硬编码色值与 `dark:` 分支（tokens.css 豁免）；确属误报（如锚点 id）可在本次写入内容任意处加 `token-guard:allow` 逃生，规则解释权在本文件。

## 核心令牌语义（实名，见 tokens.css）

| 令牌                                             | 语义                                                                                     | 用途                       |
| ------------------------------------------------ | ---------------------------------------------------------------------------------------- | -------------------------- |
| `--color-tint`                                   | 主题色（Apple 橙 `#ff9500` / `#ff9f0a`）                                                 | 强调、选中、主按钮         |
| `--color-fill` 系                                | `fill` / `secondary-fill` / `tertiary-fill` / `quaternary-fill`                          | 卡片底、次级背景、分层填充 |
| `--color-label` 系                               | `label` / `secondary-label` / `tertiary-label` / `quaternary-label` / `placeholder-text` | 文字层级                   |
| `--color-destructive`                            | 危险红                                                                                   | 删除、失败                 |
| `--color-green` / `--color-orange`               | 成功 / 警示                                                                              | 状态点、完成/等待          |
| `--color-separator` / `--color-opaque-separator` | 分割线（α / 实色）                                                                       | 边框、参考线               |
| `--color-grouped-background` 系                  | `grouped` / `secondary-grouped` / `secondary-background`                                 | iOS 分组背景层级           |
| `--shadow-card`                                  | 卡片投影                                                                                 | 唯一阴影来源               |
| `--font-sans` / `--font-mono` / `--material`     | 字体 / 材质                                                                              | 排版与材质层               |

另有完整 Apple 系统色板（`--color-blue/purple/mint/teal/…`）供徽标/分类/数据可视化使用。

## 用色规则

1. **语义色核心七族**：tint / fill 系 / label 系 / destructive / green / orange / separator；系统色板（blue/purple/mint/…）只用于徽标、分类、图表等需要区分度的场景，优先复用语义色。
2. 状态色固定映射：running=tint、done=green、waiting=orange、failed=destructive、idle=fill。全库一致，不许组件私改。
3. 数值用 `tabular-nums`（耗时、成本、token 数）。

## 动效规范（详见 styles/animations.css）

1. **一律 spring**：`ease-spring` 曲线贯穿 hover / active / 展开 / 弹层。
2. **按下即时，回弹过冲**：active 即刻响应，松手 spring 过冲模拟物理。
3. **只动 composite 属性**：transform / opacity。高度用 `grid-template-rows-[0fr]→[1fr]` 过渡实现（可中断、纯 CSS），不是 keyframes 动 height。
4. **禁止从 `scale(0)` 入场**：用 `scale(≥0.9) + opacity`；进出场时长 160–240ms。
5. 列表交错入场用既有 stagger 预设，不重写。

## 布局规则

1. **flex 优先**：`Stack`（direction/spacing/align）覆盖 90% 场景；grid 仅用于瀑布/表格等二维场景。
2. 间距走 **4px 阶梯**（gap-1/2/3/4 = 4/8/12/16），禁止魔数。
3. 圆角四档：`rounded-lg / xl / 2xl / full`。
4. 分层：卡片内容不嵌套投影（投影只有 `shadow-card` 一档）；分割用 `Divider`/`separator`，不画边框堆叠。

## 图标与文字

- 图标统一内联 SVG（lucide 线稿风格），`size` 走数字（14/16/18），描边 `strokeWidth 1.5–2`。
- 中文为主，术语与 pi 对齐（检索/合成/来源/交接/委派）；按钮文字 2-4 字。
- 组件对外文案不做硬编码业务话术，由 props 传入或使用中性默认值。

## 复用优先与反馈面（借鉴 dsh-client-ui-ux，lint 管不了的判断力）

> 来源：deepseek-harness `.agents/skills/dsh-client-ui-ux/SKILL.md`（已装 `.agents/skills/dsh-client-ui-ux/`）。写/改任何用户可见 UI 前过一遍。

### 复用优先

- 新能力先扩已有组件/容器（加 prop、加菜单项），不造平行元素堆在旁边。
- 引入任何字面值前，先找本页与最近邻的成熟组件在用什么令牌/值，直接采用；整套处理（弹层、空态、toast、页边距）同理——复用既有组件而不是发明平行的一套。
- 图标用既有 lucide 集里语义最近的；新 glyph 视为需要设计评审的例外。

### 视觉纪律

- **明暗两套必须同时可读**：降饱和/低对比色只做装饰面，禁止做正文或标签色（降饱和色在其中一个主题下必然低于可读阈值，这是固定反例）。
- **字重 ≤ 500**（组件层）：强调用字号或文字层级区分，不用 600+ 压视觉。tokens 排版专属变量是唯一例外。
- 字号从主题 typography / 本页既有 scale 取：平行内容同字号，不为单个元素发明新字号，一页 distinct 字号数要少。
- 图标、文字、相邻元素沿一条有意轴对齐（基线 / 垂直居中 / 共享边缘），图标尺寸与旁边文字成比例。

### 反馈面三选一（按消息寿命选，不按习惯选）

- **瞬态操作结果 → Toast**：toast 宿主必须比发起它的表面活得久（挂 shell 层，不挂在会被卸载的面板里）；成功与失败都报，进行中不报。
- **失败不清空数据**：删除失败保留原行，toast 报错；禁止为了展示错误把内容空白掉。
- **绑定表面的状态 → 就地 notice**：查询失败带 Retry、表单字段校验放在拥有该字段的表单里；已 populated 的面板用紧凑 notice 靠着内容放，只有空面板才居中放全页错误态。
- 错误文案：一句短白话，无内部术语；中文两句以内结尾不加句号；不得挤动相邻布局（预留空间或覆盖）。

### Loading

- 列表用骨架屏；页面级加载用居中裸 spinner，禁止塞在角落。
- 一页/一组件只有一种 loading 处理，并发区域不得各闪各的。
- 不加「加载中」文字（窄侧栏几乎从不需要）。

### Overlay 三验证（menu / popover / tooltip 必过）

1. **可关闭**：外点关闭，可聚焦的还支持 Esc。
2. **视口内**：滑动/翻转到窗口内留边距；两侧都放不下时不得振荡。
3. **不被裁剪**：逃出 overflow 祖先与 containing block（需要就 portal 到 body），不被相邻元素遮挡。

### 间距与对齐

- 改复杂页面前先建立共享的水平/垂直对齐轴，再调局部间距；平行元素对称，来历不明的一次性 gap 通常意味着漏用了共享值。
- hover / 选中 / 展开态的背景与相邻元素共享边缘与起点（如列表行 hover 背景可超出内容左缘，但内容不动）。
- 没有任何元素允许「不对齐任何东西」或明显重于邻居。
