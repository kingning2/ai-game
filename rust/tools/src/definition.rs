use serde::{Deserialize, Serialize};

/// 一份工具说明书：给 LLM / 调用方看的自描述元数据。
///
/// `parameters` 是标准 JSON Schema（draft 2020-12）对象，描述入参形状；
/// 各 LLM 协议的 function/tool 声明格式由对接层（agent 包）按此映射。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具名（调用标识；snake_case，全注册表内唯一）
    pub name: String,
    /// 一句话说明工具做什么、什么时候用（直接进 LLM prompt）
    pub description: String,
    /// 入参 JSON Schema（形如 `{"type": "object", "properties": {...}}`）
    pub parameters: serde_json::Value,
}

impl ToolDefinition {
    /// 最小可用定义：只有名字与说明，参数默认为「无参对象」
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: serde_json::json!({ "type": "object", "properties": {} }),
        }
    }

    /// 覆写参数 Schema（链式：`ToolDefinition::new(..).parameters(schema)`）
    pub fn parameters(mut self, parameters: serde_json::Value) -> Self {
        self.parameters = parameters;
        self
    }
}
