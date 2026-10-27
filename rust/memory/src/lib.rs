//! 通用 LLM 记忆系统：时间线条目模型 + LLM 压缩契约 + prompt 渲染。
//!
//! 与具体领域无关的基础能力——记忆是「主体按时间序记下的一串
//! 带标签事实」：[`AgentMemory`] 聚合、[`MemoryEntry`] 单条、
//! 自由 `tag` + 单调 `seq` 排序、[`AgentMemory::render`] 渲染。
//! 领域语义（什么算一条记忆、谁能看见）由上层编排赋予，本包不做抽取。
//!
//! ## 压缩 = LLM 编排，本包不做任何自动压缩
//!
//! 记忆变长后如何压缩是语义工作（哪些事实值得保留），交给 LLM：
//!
//! 1. [`AgentMemory::compaction_plan`]：按保留窗口划分「待压缩老段 +
//!    保留原文」，产出 [`CompactionPlan`]（含老段渲染文本，直接作 LLM 输入）
//! 2. 上层把 `plan.older_text` 交给 LLM 生成摘要
//! 3. [`AgentMemory::apply_compaction`]：把老段替换为一条摘要行
//!    （`tag = "summary"`）。plan 与当前记忆不匹配（重复应用/并发修改）
//!    返回 `false`，不会写坏记忆
//!
//! 最小用法：
//! ```
//! use memory::{AgentMemory, MemoryEntry};
//!
//! let mut memory = AgentMemory::new("agent-a");
//! for seq in 1..=6 {
//!     memory.push(MemoryEntry::new(seq, "note", None, "事实"));
//! }
//! // 只保留最近 2 条，老段（前 4 条）交给 LLM 压缩
//! let plan = memory.compaction_plan(2).expect("超窗必有方案");
//! assert_eq!(plan.older_count, 4);
//! let summary = format!("更早 {} 条的 LLM 摘要", plan.older_count);
//! assert!(memory.apply_compaction(&plan, summary));
//! assert_eq!(memory.len(), 3, "摘要行 + 保留的 2 条");
//! assert!(memory.render().contains("[#4·summary]"), "摘要锚在老段末条");
//! ```

mod transcript;

pub use transcript::{AgentMemory, CompactionPlan, MemoryEntry};
