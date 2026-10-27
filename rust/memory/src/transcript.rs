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
    /// 保留原文的条目数
    pub keep_count: usize,
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
            keep_count: keep_recent,
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
mod tests {
    use super::*;

    #[test]
    fn render_keeps_seq_and_tag() {
        let mut memory = AgentMemory::new("player-3");
        memory.push(MemoryEntry::new(1, "dawn", None, "昨夜平安，无人出局"));
        memory.push(MemoryEntry::new(1, "speech", Some("3".into()), "我是好人"));
        memory.push(MemoryEntry::new(2, "whisper", Some("1".into()), "刀 4 号"));
        let text = memory.render();
        assert!(text.contains("[#1·dawn]昨夜平安，无人出局"));
        assert!(text.contains("[#1·speech]我是好人"));
        assert!(text.contains("[#2·whisper]刀 4 号"));
    }

    #[test]
    fn sort_by_seq_is_stable() {
        let mut memory = AgentMemory::new("agent-a");
        memory.push(MemoryEntry::new(2, "note", None, "后写"));
        memory.push(MemoryEntry::new(1, "alert", None, "早"));
        memory.push(MemoryEntry::new(2, "note", None, "更后写"));
        memory.sort_by_seq();
        let seqs: Vec<u64> = memory.entries.iter().map(|entry| entry.seq).collect();
        assert_eq!(seqs, vec![1, 2, 2], "按 seq 归位，同 seq 保持追加序");
    }

    #[test]
    fn push_never_mutates_content() {
        let mut memory = AgentMemory::new("agent-p");
        memory.push(MemoryEntry::new(1, "note", None, "很长".repeat(500)));
        assert!(
            memory.entries[0].text.chars().count() == 1000,
            "push 原样保存，不做任何截断"
        );
    }

    #[test]
    fn compaction_plan_splits_older_and_keep() {
        let mut memory = AgentMemory::new("agent-c");
        for seq in 1..=5u64 {
            memory.push(MemoryEntry::new(seq, "note", None, format!("事实{seq}")));
        }
        // 未超窗：无需压缩
        assert!(memory.compaction_plan(5).is_none(), "恰好等于窗口不压缩");
        assert!(memory.compaction_plan(8).is_none(), "窗口更大不压缩");
        // 超窗：前 3 条为老段
        let plan = memory.compaction_plan(2).expect("超窗必有方案");
        assert_eq!(plan.older_count, 3);
        assert_eq!(plan.keep_count, 2);
        assert_eq!(plan.first_seq, 1);
        assert!(plan.older_text.contains("[#1·note]事实1"));
        assert!(plan.older_text.contains("[#3·note]事实3"));
        assert!(!plan.older_text.contains("事实4"), "老段不含保留条目");
        // 计划不改动记忆
        assert_eq!(memory.len(), 5, "compaction_plan 只读");
    }

    #[test]
    fn apply_compaction_replaces_older_with_summary() {
        let mut memory = AgentMemory::new("agent-d");
        for seq in 1..=5u64 {
            memory.push(MemoryEntry::new(seq, "note", None, format!("事实{seq}")));
        }
        let plan = memory.compaction_plan(2).expect("超窗必有方案");
        assert!(memory.apply_compaction(&plan, "前三天的 LLM 摘要"));
        assert_eq!(memory.len(), 3, "摘要行 + 保留 2 条");
        let first = &memory.entries[0];
        assert_eq!(first.tag, "summary", "落回的摘要行标签");
        assert_eq!(first.seq, 3, "摘要行锚在老段末条 seq（时间线不回跳）");
        assert_eq!(first.text, "前三天的 LLM 摘要");
        let rendered = memory.render();
        assert!(
            rendered.contains("[#3·summary]前三天的 LLM 摘要"),
            "{rendered}"
        );
        assert!(rendered.contains("[#5·note]事实5"), "保留原文不动");
        // 应用后可再次规划：摘要行也是普通条目，会被后续压缩一并纳入
        let next = memory.compaction_plan(1).expect("再次超窗");
        assert_eq!(next.older_count, 2, "摘要行 + 更早保留条目进入下一轮");
        assert!(next.older_text.contains("[#3·summary]"));
    }

    #[test]
    fn apply_compaction_rejects_mismatched_plan() {
        let mut memory = AgentMemory::new("agent-e");
        for seq in 1..=4u64 {
            memory.push(MemoryEntry::new(seq, "note", None, format!("事实{seq}")));
        }
        let plan = memory.compaction_plan(1).expect("超窗必有方案");
        // 重复应用：第一次已消费老段
        assert!(memory.apply_compaction(&plan, "一次"));
        assert!(!memory.apply_compaction(&plan, "二次"), "重复应用被拒绝");
        // 空方案
        let empty = CompactionPlan {
            first_seq: 1,
            older_count: 0,
            older_text: String::new(),
            keep_count: 4,
        };
        assert!(!memory.apply_compaction(&empty, "空"), "older_count=0 拒绝");
        // 越界
        let huge = CompactionPlan {
            first_seq: 1,
            older_count: 99,
            older_text: String::new(),
            keep_count: 0,
        };
        assert!(!memory.apply_compaction(&huge, "越界"), "越界拒绝");
        // 锚点不符（记忆被重算后 first_seq 变化）
        let stale = CompactionPlan {
            first_seq: 99,
            older_count: 1,
            older_text: String::new(),
            keep_count: 3,
        };
        assert!(!memory.apply_compaction(&stale, "锚错"), "锚点不符拒绝");
        assert_eq!(memory.len(), 2, "拒绝路径不改动记忆");
    }

    #[test]
    fn serde_roundtrip_is_camel_case() {
        let mut memory = AgentMemory::new("agent-b");
        memory.push(MemoryEntry::new(7, "note", Some("user".into()), "你好"));
        let json = serde_json::to_string(&memory).expect("可序列化");
        assert!(
            json.contains("\"owner\":\"agent-b\""),
            "字段应为 camelCase：{json}"
        );
        let back: AgentMemory = serde_json::from_str(&json).expect("可反序列化");
        assert_eq!(back, memory, "往返必须无损");
    }

    #[test]
    fn render_is_deterministic() {
        let mut a = AgentMemory::new("agent-f");
        a.push(MemoryEntry::new(1, "note", None, "事实"));
        let b = a.clone();
        assert_eq!(a.render(), b.render(), "同一记忆渲染必须一致");
        assert_eq!(a.len(), 1);
        assert!(!a.is_empty());
    }
}
