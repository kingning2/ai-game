#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! agent 对接包：把各家 LLM Agent 的 HTTP API 归一成统一的流式事件源。
//!
//! 分层：
//! - [`message`]：统一的请求模型（`ChatRequest`/`ChatMessage`）及其校验
//! - [`event`]：统一的流式事件模型（`AgentEvent`），即前后端 SSE 契约的 Rust 侧
//! - [`provider`]：[`AgentProvider`](provider::AgentProvider) trait + 配置加载 +
//!   [`build_provider`](provider::build_provider) 工厂
//! - [`openai`] / [`anthropic`]：两家协议实现（协议差异在各自文件内消化）
//! - [`sse`] / [`stream`]：SSE 解码 + 逐帧变换（协议实现共用）
//! - [`error`]：[`AgentError`](error::AgentError) 类型与构造策略（文案 / 截断）
//!
//! 最小用法：
//! ```no_run
//! use agent::message::ChatMessage;
//! use agent::provider::{AgentConfig, AgentProvider, build_provider};
//!
//! # async fn demo() -> agent::AgentResult<()> {
//! // 配置来自前端设置页（server 持久化到配置文件），这里用默认值演示
//! let provider = build_provider(&AgentConfig::default())?;
//!
//! let mut stream = provider
//!     .chat_stream(agent::message::ChatRequest {
//!         messages: vec![ChatMessage::user("你好")],
//!         ..agent::message::ChatRequest::default()
//!     })
//!     .await?;
//!
//! // pull 式消费：Ok(None) = 正常结束，Err = 流中断 / 协议错误
//! while let Some(event) = stream.next_event().await? {
//!     if let agent::event::AgentEvent::TextDelta { delta } = event {
//!         print!("{delta}");
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod anthropic;
pub mod error;
pub mod event;
pub mod message;
pub mod openai;
pub mod provider;
pub mod sse;
pub mod stream;

// 错误类型定义在 [`error`] 模块，包根 re-export 保持原路径可用
pub use error::{AgentError, AgentResult};
pub use message::{ChatMessage, ChatRequest, ThinkingMode, ToolCall, ToolDefinition, ToolResult};
// 包根主要类型出口：对接层的使用方（state/server）不需要进模块路径
pub use provider::{
    AgentConfig, AgentProvider, ChatStreamFuture, ProviderKind, ProviderSettings, build_provider,
};
pub use stream::ChatStream;

#[cfg(test)]
mod tests;
