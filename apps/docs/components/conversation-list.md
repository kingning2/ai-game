# ConversationList 会话列表

多会话管理入口：自动时间分组（今天/最近 7 天/更早）+ 当前会话高亮 + 行内重命名 + 删除 + 新建。

## 何时使用

- Agent 控制台侧边栏：历史会话罗列与切换
- `activeId` 受控高亮；选择/重命名/删除都只发回调，数据归调用方
- 传 `date` 自动分三组；不传则平铺（无分组标题）
- 重命名是行内 input：Enter/失焦提交、Esc 取消

## 代码演示

<DemoPreview name="ConversationListBasic" />

<<< ../demos/ConversationList/basic.tsx

## API

### Conversation

| 属性    | 说明                                   | 类型     |
| ------- | -------------------------------------- | -------- |
| id      | 会话 id                                | `string` |
| title   | 会话标题                               | `string` |
| preview | 最近一条消息预览（一行截断）           | `string` |
| time    | 展示时间，如 `'14:32'`/`'昨天'`        | `string` |
| date    | 会话时间，驱动自动分组；不传归入"更早" | `Date`   |

### ConversationListProps

| 属性          | 说明                       | 类型                                  |
| ------------- | -------------------------- | ------------------------------------- |
| conversations | 会话数组                   | `Conversation[]`                      |
| activeId      | 当前激活会话 id（受控）    | `string`                              |
| onSelect      | 选中回调                   | `(id: string) => void`                |
| onRename      | 重命名回调                 | `(id: string, title: string) => void` |
| onDelete      | 删除回调                   | `(id: string) => void`                |
| onNew         | 新建按钮回调（传入才显示） | `() => void`                          |

## 设计细节

- **hover 才出现的操作按钮**：纯 CSS opacity（`group-hover` + `focus-within`），无 JS 开销，键盘可达
- **重命名/删除按钮**：删除 hover 转 destructive 色；`stopPropagation` 防止触发选中
- **active 行**：`bg-tint/10` + 标题加粗；键盘操作 Enter/Space 可选中（`role="button"`）
- **组件内唯一状态是 UI 态**（editingId/draft），会话数据永远归调用方
