# Usage 用量

AI 用量展示套件，包含三个导出：`UsageMeter` 上下文窗口占用条（70% 转橙、90% 转红）、`UsageBadge` 输入 / 输出 token 徽章、`formatTokens` token 数格式化纯函数。胶囊形等宽数字，对齐稳定。

## 何时使用

- 对话界面展示**上下文窗口占用**，接近上限时颜色自动预警（70% 橙、90% 红）
- 每条 AI 回复下方挂 `UsageBadge`，用 ↑ 输入 / ↓ 输出 直观呈现单次 token 消耗
- 自定义面板、统计报表里需要把 token 数缩写为 `1.2K` / `3.4M` 时，直接调用 `formatTokens` 纯函数
- 三个导出可独立使用，也可在同一卡片内组合

## 代码演示

### 占用条三档色阶

`used / total` 低于 70% 主题色、70–90% 橙色、达到 90% 红色，无需业务判断阈值。

<DemoPreview name="UsageMeterBasic" />

<<< ../demos/Usage/meter-basic.tsx

### 实时增长

对话推进时 `used` 持续增长，进度与文字同步过渡。

<DemoPreview name="UsageMeterLive" />

<<< ../demos/Usage/meter-live.tsx

### 用量徽章

`prompt`（↑）与 `completion`（↓）任意组合；两项都不传时整体渲染 `null`。

<DemoPreview name="UsageBadgeBasic" />

<<< ../demos/Usage/badge-basic.tsx

### 卡片内组合

徽章看单次消耗、占用条看窗口总量，一条消息的用量全貌。

<DemoPreview name="UsageCombined" />

<<< ../demos/Usage/combined.tsx

### formatTokens 格式化

纯函数导出：千位缩写为 K、百万位缩写为 M，自动去尾零（`1200 → 1.2K`、`1000 → 1K`）。

<DemoPreview name="UsageFormatTokens" />

<<< ../demos/Usage/format-tokens.tsx

## API

### UsageMeterProps

| 属性        | 说明                     | 类型      | 默认值     |
| ----------- | ------------------------ | --------- | ---------- |
| used        | 已用 tokens              | `number`  | -          |
| total       | 上下文窗口总量           | `number`  | -          |
| label       | 左侧标签文案             | `string`  | `'上下文'` |
| showPercent | 是否在数值后附带百分比   | `boolean` | `true`     |
| className   | 附加类名，可覆盖内部样式 | `string`  | -          |

### UsageBadgeProps

| 属性       | 说明                                    | 类型     | 默认值 |
| ---------- | --------------------------------------- | -------- | ------ |
| prompt     | 输入（提示词）tokens，不传则不显示 ↑ 项 | `number` | -      |
| completion | 输出（回复）tokens，不传则不显示 ↓ 项   | `number` | -      |
| className  | 附加类名，可覆盖内部样式                | `string` | -      |

### formatTokens

```ts
function formatTokens(n: number): string;
```

| 参数 | 说明                                     | 类型     |
| ---- | ---------------------------------------- | -------- |
| n    | token 数量（支持负数，按绝对值判断档位） | `number` |

返回千位以上缩写（`1234 → "1.2K"`、`1520000 → "1.5M"`），不足 1000 原样返回；小数一位，尾零自动去除（`1000 → "1K"`）。

### 色阶规则

| 占用比例  | 进度条         | 百分比文字 |
| --------- | -------------- | ---------- |
| < 70%     | 主题色（tint） | 三级文字色 |
| 70% – 90% | 橙色           | 橙色       |
| ≥ 90%     | 红色           | 红色       |

## 设计细节

- **等宽数字**：徽章与数值均用 `font-mono tabular-nums`，实时刷新时数字不跳动
- **双段色阶预警**：阈值判断内置在组件里（`ratio >= 0.9 → red`、`>= 0.7 → orange`），颜色与文字同步变化
- **无障碍**：占用条渲染 `role="progressbar"` 并输出 `aria-valuenow`（百分比），屏幕阅读器直接播报
- **零渲染保护**：`UsageBadge` 在 prompt / completion 都缺省时返回 `null`，列表里可以放心按需挂载
