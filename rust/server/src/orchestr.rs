//! 对局编排：跨包组合 game 引擎与 Agent 基础包，产出决策上下文。
//!
//! 子包只提供基础能力（game = 规则引擎，memory = 记忆模型，agent = LLM 协议）；
//! 「从引擎事件流抽取某玩家视角的记忆」同时消费两个域的行为，按分层约定
//! 收敛在 server 编排层。每个子模块一个编排能力：
//!
//! - [`memory`]：[`recall`](memory::recall)——玩家视角记忆抽取

pub mod autoplay;
pub mod memory;
