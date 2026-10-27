# MemoryPanel 记忆面板

Agent 的持久记忆面板：记忆按类型分档（fact 事实 / note 备注 / entity 实体 / plan 计划），带时间与置顶，支持检索。数据模型对齐 pi 的 para-memory-files 记忆体系。

## 何时使用

- 展示 agent 跨会话记住的事实、计划与实体关系
- 检索框可选：传 `query` / `onQueryChange` 走受控；都不传则组件内过滤（纯展示逻辑）
- 置顶项固定在列表最前，始终不参与过滤排序

## 代码演示

### 基础用法

五种记忆条目（含一条置顶事实）；在搜索框输入关键词体验框内过滤。

<DemoPreview name="MemoryPanelBasic" />

<<< ../demos/MemoryPanel/basic.tsx

## API

### MemoryPanelProps

| 属性            | 类型                           | 默认值   | 说明                                  |
| --------------- | ------------------------------ | -------- | ------------------------------------- |
| `entries`       | `MemoryEntry[]`                | —        | 记忆条目                              |
| `title`         | `string`                       | `'记忆'` | 面板标题                              |
| `query`         | `string`                       | —        | 受控搜索词（与 `onQueryChange` 搭配） |
| `onQueryChange` | `(q: string) => void`          | —        | 搜索词变更回调                        |
| `onOpenEntry`   | `(entry: MemoryEntry) => void` | —        | 条目点击回调                          |
| `searchable`    | `boolean`                      | `true`   | 是否显示检索框                        |
| `className`     | `string`                       | —        | 追加到根元素的类名                    |

### MemoryEntry

| 属性      | 类型       | 说明                               |
| --------- | ---------- | ---------------------------------- |
| `id`      | `string`   | 唯一 id                            |
| `kind`    | `'fact' \\ | 'note' \\                          | 'entity' \\ | 'plan'` | 类型徽标：fact=blue / plan=orange / entity=tint / note=灰 |
| `content` | `string`   | 记忆内容                           |
| `time`    | `string?`  | 记录时间，如 `'昨天 14:20'`        |
| `pinned`  | `boolean?` | 置顶：固定在列表最前，不受过滤影响 |
