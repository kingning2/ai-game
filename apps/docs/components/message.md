# Message 消息 / MessageList 消息流

对话骨架：`Message` 渲染单条消息（混合式布局 —— user 气泡右对齐、assistant 通栏），`MessageList` 提供滚动容器（贴底自动跟随 + 回到底部按钮）。

## 何时使用

- Agent 对话主界面：把角色排布、loading 态、消息操作条标准化，不再每页手搓
- user 消息用 tint 气泡（iMessage 式）；assistant 消息通栏展示，长 Markdown/工具卡不受气泡挤压
- `loading` 态 = 三点呼吸，用于 assistant 等待首 token
- 组件零业务逻辑：复制/重新生成只发回调，消息数据归调用方

## 代码演示

### 基础用法

混合式布局 + loading 态 + 操作条。

<DemoPreview name="MessageBasic" />

<<< ../demos/Message/basic.tsx

### 滚动跟随

贴底时新内容自动跟随；上滑后出现"回到底部"按钮。

<DemoPreview name="MessageScroll" />

<<< ../demos/Message/scroll.tsx

## API

### MessageProps

| 属性         | 说明                                | 类型                    | 默认值  |
| ------------ | ----------------------------------- | ----------------------- | ------- |
| role         | 消息角色                            | `'user' \| 'assistant'` | -       |
| name         | assistant 头部名字                  | `string`                | `'ARK'` |
| timestamp    | 头部时间戳，如 `'14:32'`            | `string`                | -       |
| loading      | assistant 等待首 token 的三点呼吸态 | `boolean`               | `false` |
| onCopy       | 复制回调（传入即显示复制按钮）      | `() => void`            | -       |
| onRegenerate | 重新生成回调（传入即显示）          | `() => void`            | -       |
| className    | 附加类名                            | `string`                | -       |
| children     | 消息内容；loading 态下可省略        | `ReactNode`             | -       |

### MessageListProps

| 属性           | 说明                                 | 类型     | 默认值 |
| -------------- | ------------------------------------ | -------- | ------ |
| stickThreshold | 距底部小于该值(px)视为贴底，自动跟随 | `number` | `48`   |
| jumpThreshold  | 距底部超过该值(px)显示回到底部按钮   | `number` | `160`  |
| className      | 附加类名（父容器需有确定高度）       | `string` | -      |

## 设计细节

- **贴底判定用 ref 而非 state**：children 更新的 effect 里直接读 ref，不吃 state 滞后一帧导致的"跟丢"
- **回到底部按钮显隐双路径**：Chrome 133+ 用 `scroll-state()` 容器查询在 CSS 层驱动（`scrollable: bottom` 未贴底时显示，滚动零重渲染）；其他引擎降级为滚动监听 + setState，跟随逻辑（stickRef）两条路径共用
- **回到底部按钮**：绝对定位在容器底部，`pointer-events-none` 外层；隐藏时按钮同步不可点（透明命中修复），显隐用 opacity 过渡
- **滚动穿透**：容器加 `overscroll-contain`，滚动到头不带动页面
- **user 气泡**：`rounded-2xl rounded-br-md bg-tint text-white`，`max-w-[80%]`
- **操作条**：`size-7` 图标按钮，tertiary 色低调常驻（移动端没有 hover，不做成 hover 才出现）
