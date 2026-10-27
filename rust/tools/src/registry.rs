use serde_json::Value;

use crate::definition::ToolDefinition;
use crate::error::ToolError;
use crate::tool::Tool;

/// 工具注册表：按名收集工具，批量导出说明书，按名查找 / 调用。
///
/// 对接层用法：`definitions()` 的结果交给 LLM 协议层映射成 tools 声明；
/// LLM 回来点名要调的工具后，用 `execute(name, args)` 派发。
#[derive(Default)]
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    /// 空注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个工具；重名直接拒绝（说明书批量导出后名字必须可作唯一键）
    pub fn register(&mut self, tool: Box<dyn Tool>) -> Result<(), ToolError> {
        let name = tool.definition().name;
        if self.tools.iter().any(|t| t.definition().name == name) {
            return Err(ToolError::DuplicateTool { name });
        }
        self.tools.push(tool);
        Ok(())
    }

    /// 全部工具的说明书（按注册顺序；LLM prompt 里的稳定性靠注册顺序保证）
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|t| t.definition()).collect()
    }

    /// 按名查找
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|t| t.definition().name == name)
            .map(|t| t.as_ref())
    }

    /// 按名派发一次调用；未知工具报 [`ToolError::UnknownTool`]
    pub fn execute(&self, name: &str, args: Value) -> Result<Value, ToolError> {
        self.get(name).map_or_else(
            || {
                Err(ToolError::UnknownTool {
                    name: name.to_string(),
                })
            },
            |tool| tool.execute(args),
        )
    }
}
