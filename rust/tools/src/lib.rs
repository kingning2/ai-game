#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! 工具暴露基座：把「一个可供 LLM / 调用方调用的动作」抽象成统一契约。
//!
//! ## 定位
//!
//! 与具体领域无关的基础设施——一个工具 = 一份自描述的
//! [`ToolDefinition`]（名字 / 说明 / JSON Schema 参数）+ 一次
//! [`Tool::execute`]（JSON 入参 → JSON 出参）。
//! 领域语义（工具代表什么动作、谁能调用）由领域包实现 [`Tool`] 赋予，
//! 比如狼人杀侧把「狼刀 / 女巫用药」各包成一个工具（见 `games/werewolf/src/tools/`）。
//!
//! 分层：
//! - [`definition`]：[`ToolDefinition`](definition::ToolDefinition)——给 LLM 看的工具说明书
//! - [`error`]：[`ToolError`](error::ToolError)——未知工具 / 参数不合法 / 执行失败三分类
//! - [`args`]：[`Args`](args::Args)——参数提取小帮手（类型化取字段，失败自动归到 InvalidArguments）
//! - [`tool`]：[`Tool`](tool::Tool)——基类 trait，实现方只需提供定义与执行
//! - [`registry`]：[`ToolRegistry`](registry::ToolRegistry)——按名收集 / 查找 / 调用，批量导出定义
//!
//! ## 最小用法
//!
//! ```
//! use serde_json::json;
//! use tools::{Args, Tool, ToolDefinition, ToolError, ToolRegistry};
//!
//! struct EchoTool;
//!
//! impl Tool for EchoTool {
//!     fn definition(&self) -> ToolDefinition {
//!         ToolDefinition::new("echo", "原样返回输入文本")
//!             .parameters(json!({"type": "object", "properties": {"text": {"type": "string"}}, "required": ["text"]}))
//!     }
//!
//!     fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value, ToolError> {
//!         let args = Args::parse("echo", args)?;
//!         Ok(json!({ "echo": args.string("text")? }))
//!     }
//! }
//!
//! let mut registry = ToolRegistry::new();
//! registry.register(Box::new(EchoTool)).unwrap();
//!
//! // 给 LLM 的工具说明书直接导出（未来 agent 包按各家协议映射）
//! let defs = registry.definitions();
//! assert_eq!(defs.len(), 1);
//! assert_eq!(defs[0].name, "echo");
//!
//! // 按名调用；未知工具 / 坏参数统一走 ToolError
//! let out = registry.execute("echo", json!({ "text": "你好" })).unwrap();
//! assert_eq!(out, json!({ "echo": "你好" }));
//! ```

pub mod args;
pub mod definition;
pub mod error;
pub mod registry;
pub mod tool;

pub use args::Args;
pub use definition::ToolDefinition;
pub use error::ToolError;
pub use registry::ToolRegistry;
pub use tool::Tool;

#[cfg(test)]
mod tests;
