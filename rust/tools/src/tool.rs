use crate::definition::ToolDefinition;
use crate::error::ToolError;

/// 工具基类：一个可被 LLM / 调用方调用的动作。
///
/// 实现方只需两件事：
/// 1. [`definition`](Tool::definition)——自描述的说明书（名字 / 说明 / 参数 Schema）
/// 2. [`execute`](Tool::execute)——JSON 入参 → JSON 出参；领域失败折叠进 [`ToolError::Execution`]
///
/// 约定：实现是纯同步的（领域引擎如狼人杀是同步状态机）；需要 IO 的工具
/// 由调用方在外层起任务，本 trait 不引入 async 以保持对象安全与简单。
pub trait Tool: Send + Sync {
    /// 工具说明书
    fn definition(&self) -> ToolDefinition;

    /// 执行一次调用。入参保证是 JSON 对象（实现内用 [`Args`](crate::Args) 逐字段取）
    fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value, ToolError>;
}
