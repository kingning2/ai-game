//! 通用 LLM 记忆系统：时间线条目模型 + LLM 压缩契约 + prompt 渲染。
//!
//! 与任何具体领域（对局/客服/巡检…）无关的基础能力：记忆是
//! 「主体（owner）按时间序记下的一串带标签事实」，可渲染成
//! LLM 决策 prompt 的事实段。标签是自由字符串，领域语义由
//! 上层编排赋予——本包不做任何领域抽取。
//!
//! ## 压缩是显式的 LLM 编排，本包不做任何自动压缩
//!
//! 「哪些记忆值得保留」是语义判断，机械截断/计数折叠都会丢真话。
//! 因此压缩拆成三步（见 [`crate`]-level 文档）：
//! [`AgentMemory::compaction_plan`] 划分 → 上层调 LLM 生成摘要 →
//! [`AgentMemory::apply_compaction`] 落回。`push` 永远原样追加，
//! 记忆内容只被显式的压缩动作改变。
//!
//! 纯数据、确定性（同一组条目 ⇒ 同一记忆与同一划分）。

use serde::{Deserialize, Serialize};

/// 一条通用记忆：单调序号 + 自由标签 + 来源 + 内容。
///
/// `seq` 由调用方定义单调时钟（如递增序号、业务时间片），
/// [`AgentMemory::sort_by_seq`] 据此归位；`tag` 是自由标签
/// （如 `"note"` / `"alert"` / `"summary"`，领域自定）；`source`
/// 是记录者标识。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntry {
    /// 单调序号（时间线排序键）
    pub seq: u64,
    /// 自由领域标签
    pub tag: String,
    /// 记录者标识（系统记录为 `None`）
    pub source: Option<String>,
    /// 事实内容（原样保存，压缩只经 [`AgentMemory::apply_compaction`])
    pub text: String,
}

impl MemoryEntry {
    /// 便捷构造
    pub fn new(
        seq: u64,
        tag: impl Into<String>,
        source: Option<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            seq,
            tag: tag.into(),
            source,
            text: text.into(),
        }
    }
}

/// 压缩方案：把记忆分成「待 LLM 压缩的老段 + 保留原文的新段」。
///
/// 由 [`AgentMemory::compaction_plan`] 产出，`older_text` 直接作为
/// LLM 输入；[`AgentMemory::apply_compaction`] 按方案落回摘要。
/// 字段快照化（含 `first_seq`）以便应用时校验记忆未被并发改动。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactionPlan {
    /// 待压缩段最旧条目的 seq（应用时校验锚点）
    pub first_seq: u64,
    /// 待压缩条目数（前缀长度）
    pub older_count: usize,
    /// 待压缩段的渲染文本（直接作 LLM 输入）
    pub older_text: String,
}

/// 一个主体的完整记忆：按时间序的事实流水
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentMemory {
    /// 记忆主体标识（座位号 / 会话 id / agent 名，领域自定）
    pub owner: String,
    /// 按时间序的事实流水
    pub entries: Vec<MemoryEntry>,
}

impl AgentMemory {
    /// 空记忆
    pub fn new(owner: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            entries: Vec::new(),
        }
    }

    /// 原样追加一条事实（不做任何压缩；压缩见 [`Self::compaction_plan`]）
    pub fn push(&mut self, entry: MemoryEntry) {
        self.entries.push(entry);
    }

    /// 划分压缩方案：最旧 `len - keep_recent` 条为老段，其余保留原文。
    ///
    /// 未超窗（`len <= keep_recent`）返回 `None`——无需压缩，上层
    /// 不必调 LLM。`keep_recent == 0` 表示全部可压缩（记忆清空，
    /// 仅剩一条摘要）。
    pub fn compaction_plan(&self, keep_recent: usize) -> Option<CompactionPlan> {
        if self.entries.len() <= keep_recent {
            return None;
        }
        let older_count = self.entries.len() - keep_recent;
        let older = &self.entries[..older_count];
        let older_text = older
            .iter()
            .map(|entry| format!("[#{}·{}]{}", entry.seq, entry.tag, entry.text))
            .collect::<Vec<_>>()
            .join("\n");
        Some(CompactionPlan {
            first_seq: older[0].seq,
            older_count,
            older_text,
        })
    }

    /// 应用压缩结果：把方案的老段替换为一条 LLM 摘要行。
    ///
    /// 摘要行 `tag = "summary"`、`seq` 锚在老段末条（摘要覆盖截至
    /// 该时刻的事实，时间线不回跳）。方案与当前记忆不匹配
    /// （`older_count` 越界或首条 `seq` 对不上——重复应用 / 并发修改 /
    /// 记忆已重算）返回 `false` 且不改动记忆。
    pub fn apply_compaction(&mut self, plan: &CompactionPlan, summary: impl Into<String>) -> bool {
        if plan.older_count == 0
            || plan.older_count > self.entries.len()
            || self.entries[0].seq != plan.first_seq
        {
            return false;
        }
        let first_seq = self.entries[plan.older_count - 1].seq;
        let summary_entry = MemoryEntry {
            seq: first_seq,
            tag: "summary".to_string(),
            source: None,
            text: summary.into(),
        };
        self.entries.drain(..plan.older_count);
        self.entries.insert(0, summary_entry);
        true
    }

    /// 按 `seq` 稳定排序（同 seq 条目保持追加序）
    pub fn sort_by_seq(&mut self) {
        self.entries.sort_by_key(|entry| entry.seq);
    }

    /// 条目数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 拼成可读文本，直接作为 LLM 决策 prompt 的事实段
    pub fn render(&self) -> String {
        self.entries
            .iter()
            .map(|entry| format!("[#{}·{}]{}", entry.seq, entry.tag, entry.text))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
#[path = "transcript_tests.rs"]
mod tests;
