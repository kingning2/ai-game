# UI 设计规范（Apple HIG · 令牌驱动）

> 本文件覆盖 `rules/common/` 中与 UI 相关的约定。设计理念版见 `apps/docs/guide/design.md`。

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
