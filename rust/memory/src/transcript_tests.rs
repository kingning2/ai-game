//! transcript.rs 的单元测试（由内联 mod tests 迁入）。

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
    };
    assert!(!memory.apply_compaction(&empty, "空"), "older_count=0 拒绝");
    // 越界
    let huge = CompactionPlan {
        first_seq: 1,
        older_count: 99,
        older_text: String::new(),
    };
    assert!(!memory.apply_compaction(&huge, "越界"), "越界拒绝");
    // 锚点不符（记忆被重算后 first_seq 变化）
    let stale = CompactionPlan {
        first_seq: 99,
        older_count: 1,
        older_text: String::new(),
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
