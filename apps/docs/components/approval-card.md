# ApprovalCard 授权确认

Agent 请求执行敏感操作（删文件/花钱/外发数据）时的授权卡：操作描述 + 允许/总是允许/拒绝，人机协同必备。

## 何时使用

- agent 要执行高风险操作前先请求授权；`pending` 显示按钮组，批准/拒绝后显示结果行
- 状态由调用方驱动：`onApprove/onAlwaysAllow/onDeny` 只发回调，是否"记住选择"由上层决定
- `note` 用于批准后的补充说明（如"本次会话内不再询问"）

## 代码演示

### 状态流转

pending → approved/denied，note 随操作不同。

<DemoPreview name="ApprovalCardBasic" />

<<< ../demos/ApprovalCard/basic.tsx

### 结果态

approved / denied 静态展示。

<DemoPreview name="ApprovalCardSettled" />

<<< ../demos/ApprovalCard/settled.tsx

## API

### ApprovalRequest

| 属性        | 说明                                  | 类型     |
| ----------- | ------------------------------------- | -------- |
| title       | 操作标题，如 `删除文件 src/legacy.ts` | `string` |
| description | 操作详情/风险说明                     | `string` |
| kind        | 操作类型徽标：`bash`/`file`/`network` | `string` |

### ApprovalCardProps

| 属性          | 说明                       | 类型                                  | 默认值      |
| ------------- | -------------------------- | ------------------------------------- | ----------- |
| request       | 操作描述对象               | `ApprovalRequest`                     | -           |
| status        | 授权状态                   | `'pending' \| 'approved' \| 'denied'` | `'pending'` |
| onApprove     | 允许一次                   | `() => void`                          | -           |
| onAlwaysAllow | 总是允许（传入才显示按钮） | `() => void`                          | -           |
| onDeny        | 拒绝                       | `() => void`                          | -           |
| note          | 结果态补充说明             | `string`                              | -           |

## 设计细节

- **警示色用 orange**（Apple 语义里的"警告"），拒绝态图标转 red
- **按钮规范**：允许=solid 主操作、总是允许=tinted 次确认、拒绝=plain 覆盖 destructive 文字色
- **kind 徽标**：tertiary-fill 胶囊，让操作类型一眼可辨
