//! mvp.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

fn voter(name: &str, seat: u8) -> Voter {
    Voter {
        seat,
        name: name.to_string(),
        role: "villager".to_string(),
    }
}

#[test]
fn parse_vote_accepts_seat_number_and_name() {
    let roster = vec![
        (1, "甲".to_string()),
        (2, "乙".to_string()),
        (3, "丙".to_string()),
    ];
    let me = voter("甲", 1);
    // 座位号（JSON 字符串包裹）
    let (cand, reason) =
        parse_vote(r#"{"vote":"3","reason":"带领好人翻盘"}"#, &me, &roster).unwrap();
    assert_eq!(cand, "丙");
    assert_eq!(reason, "带领好人翻盘");
    // 名字
    let (cand, _) = parse_vote(r#"{"vote":"乙","reason":"发言清晰"}"#, &me, &roster).unwrap();
    assert_eq!(cand, "乙");
    // 座位号带后缀（「3号」）
    let (cand, _) = parse_vote(r#"{"vote":"3号","reason":"狼刀稳准"}"#, &me, &roster).unwrap();
    assert_eq!(cand, "丙");
}

#[test]
fn parse_vote_rejects_self_and_unknown_and_noise() {
    let roster = vec![(1, "甲".to_string()), (2, "乙".to_string())];
    let me = voter("甲", 1);
    // 不能投自己
    assert!(parse_vote(r#"{"vote":"甲","reason":""}"#, &me, &roster).is_none());
    assert!(parse_vote(r#"{"vote":"1","reason":""}"#, &me, &roster).is_none());
    // 名册外
    assert!(parse_vote(r#"{"vote":"9","reason":""}"#, &me, &roster).is_none());
    // 无 JSON / 无 vote 字段
    assert!(parse_vote("我觉得丙不错", &me, &roster).is_none());
    assert!(parse_vote(r#"{"reason":"只有理由"}"#, &me, &roster).is_none());
}

#[test]
fn parse_vote_extracts_json_from_markdown_fence() {
    let roster = vec![(1, "甲".to_string()), (2, "乙".to_string())];
    let me = voter("甲", 1);
    let reply = "好的，我的投票如下：\n```json\n{\"vote\":\"2\",\"reason\":\"逻辑最强\"}\n```";
    let (cand, reason) = parse_vote(reply, &me, &roster).unwrap();
    assert_eq!(cand, "乙");
    assert_eq!(reason, "逻辑最强");
}

#[test]
fn truncate_limits_chars_without_panic() {
    assert_eq!(truncate("abcdef", 3), "abc\n…（后续事件略）");
    assert_eq!(truncate("短的", 10), "短的");
    // 多字节字符按 char 截断
    let long = "狼".repeat(50);
    assert_eq!(truncate(&long, 10).chars().count(), 10 + 9);
}
