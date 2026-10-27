# SettingsPanel 设置面板

Agent 设置面板：前端配置 API Key 与连接参数的表单载体，配合后端 `PUT /api/agent/config` 使用。Key 输入框只显示服务端脱敏后的占位（如 `****cdef`），明文从服务端回不到界面；留空提交 = 保留现有 Key。

## 何时使用

- 应用需要用户自行填写模型服务的 API Key / Base URL / 模型名（OpenAI 兼容 + Anthropic 双协议）
- 与后端「agent 配置端点」配套：`GET` 拉取当前配置初始化草稿，`PUT` 保存后热生效
- 敏感信息表单：明文只在输入框存在，回显一律脱敏
- 切换供应商协议时，地址与模型自动切回该协议当前生效值，Key 始终置空待填

## 代码演示

### 基础用法

组件零内部表单状态：`draft` + `onDraftChange` 受控，保存交给 `onSave` 回调（demo 用 1s 延迟模拟请求）。

<DemoPreview name="SettingsPanelBasic" />

<<< ../demos/SettingsPanel/basic.tsx

## API

### SettingsPanelProps

| 属性          | 说明                                                          | 类型                                  | 默认值  |
| ------------- | ------------------------------------------------------------- | ------------------------------------- | ------- |
| value         | 当前生效配置（决定占位文案与「已配置」状态）                  | `AgentSettingsValue`                  | -       |
| draft         | 受控草稿（当前编辑中的一组参数）                              | `AgentSettingsDraft`                  | -       |
| onDraftChange | 草稿变更回调；切换供应商时组件会把地址/模型重置为该协议当前值 | `(draft: AgentSettingsDraft) => void` | -       |
| onSave        | 保存回调，由调用方接 `PUT /api/agent/config`                  | `(draft: AgentSettingsDraft) => void` | -       |
| saving        | 保存请求进行中（按钮禁用 + 文案切换）                         | `boolean`                             | `false` |
| error         | 保存失败的错误文案（红字提示行）                              | `string \| null`                      | `null`  |
| disabled      | 整体禁用                                                      | `boolean`                             | `false` |
| className     | 附加类名，可覆盖内部样式                                      | `string`                              | -       |

### AgentSettingsValue

| 字段      | 说明                            | 类型                      |
| --------- | ------------------------------- | ------------------------- |
| provider  | 当前生效供应商                  | `'openai' \| 'anthropic'` |
| openai    | OpenAI 兼容协议的连接参数展示态 | `ProviderSettingsView`    |
| anthropic | Anthropic 协议的连接参数展示态  | `ProviderSettingsView`    |
| maxTokens | 单次回复 Token 上限             | `number`                  |

### ProviderSettingsView

| 字段         | 说明                                | 类型      |
| ------------ | ----------------------------------- | --------- |
| baseUrl      | API 根地址                          | `string`  |
| model        | 默认模型                            | `string`  |
| apiKeyMasked | 服务端脱敏后的 Key（如 `****cdef`） | `string`  |
| hasKey       | 是否已配置 Key（决定占位文案）      | `boolean` |

### AgentSettingsDraft

| 字段      | 说明                                                                | 类型                      |
| --------- | ------------------------------------------------------------------- | ------------------------- |
| provider  | 正在编辑的供应商                                                    | `'openai' \| 'anthropic'` |
| baseUrl   | API 根地址                                                          | `string`                  |
| model     | 默认模型                                                            | `string`                  |
| apiKey    | 新输入的 Key；**空串 = 保持现有 Key 不变**（对应后端 PUT 缺省语义） | `string`                  |
| maxTokens | 单次回复 Token 上限                                                 | `number`                  |

## 设计细节

- **明文不回流**：服务端只返回 `apiKeyMasked`（`****` + 末 4 位），输入框用占位文案展示「已配置」状态；Key 输入框 `type="password"` + `autoComplete="off"`
- **缺省即保留**：草稿的 `apiKey` 空串在保存时不下发，后端保留现有 Key——改地址/模型不用重填密钥
- **协议隔离**：SegmentedControl 切换供应商时重置草稿为该协议当前生效值，避免把 OpenAI 的地址误存到 Anthropic
- **受控纯组件**：不持有表单状态、不感知请求细节，请求与持久化全部由调用方编排
