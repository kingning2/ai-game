# SkillCard 技能卡片

展示 Agent 可用的技能（skill）：图标（默认 sparkle）+ 名称 + 描述 + 可选分类标签，右侧可选启用开关。纯展示组件，零业务逻辑。

## 何时使用

- Agent 控制台的「技能」面板，罗列当前可用的技能清单（名称 + 一句话说明 + 分类）
- 需要让用户逐项启停技能时，传入 `onToggle` 即显示 iOS 风格 Switch；不传则是纯展示
- 组件不持有开关状态：`enabled` 由调用方传入、切换结果通过 `onToggle` 回调，重渲染/持久化都在调用方

## 代码演示

### 基础用法

技能网格（二维布局才用 grid），纯展示、不带开关；描述超两行自动截断。

<DemoPreview name="SkillCardBasic" />

<<< ../demos/SkillCard/basic.tsx

### 启用开关

传入 `onToggle` 显示右侧 Switch，状态由调用方 useState 驱动。

<DemoPreview name="SkillCardToggle" />

<<< ../demos/SkillCard/toggle.tsx

## API

### SkillCardProps

| 属性        | 说明                                       | 类型                         | 默认值  |
| ----------- | ------------------------------------------ | ---------------------------- | ------- |
| name        | 技能名，如 `arkcli-billing` / `web-search` | `string`                     | -       |
| description | 技能描述（做什么、何时触发），两行截断     | `string`                     | -       |
| tags        | 分类标签，胶囊样式                         | `string[]`                   | -       |
| icon        | 自定义图标；不传用默认 sparkle             | `ReactNode`                  | -       |
| enabled     | 是否启用（受控）；配合 onToggle 展示开关   | `boolean`                    | `false` |
| onToggle    | 切换启用回调；传入即显示开关               | `(enabled: boolean) => void` | -       |
| className   | 附加类名，可覆盖内部样式                   | `string`                     | -       |

## 设计细节

- **图标位**：`size-9` 圆角方块 + `bg-tint/10 text-tint`，默认 sparkle 图标，可用 `icon` 换成任意 ReactNode
- **文案层级**：名称 `text-sm font-semibold text-label`，描述 `text-xs text-secondary-label line-clamp-2`
- **标签**：令牌胶囊（`bg-tertiary-fill` 圆角小片），自动换行不撑破卡片
- **开关**：复用基础 `Switch`（iOS 规格），`aria-label` 自动带上技能名
- **容器**：`rounded-xl bg-secondary-grouped-background shadow-card`，与 ThinkingBlock / ToolExecution 同一套卡片语言
