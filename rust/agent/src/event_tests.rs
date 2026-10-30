//! `event.rs` 的单元测试。

use super::*;
use crate::message::ToolCall;

/// serde tag 约定：server 的 SSE 转发直接 to_string，字段名变更要过这里
#[test]
fn event_serde_wire_format() {
    let text = serde_json::to_string(&AgentEvent::TextDelta {
        delta: "你好".into(),
    })
    .unwrap();
    assert_eq!(text, r#"{"type":"text_delta","delta":"你好"}"#);

    let done = serde_json::to_string(&AgentEvent::Done {
        stop_reason: StopReason::EndTurn,
    })
    .unwrap();
    assert_eq!(done, r#"{"type":"done","stop_reason":"end_turn"}"#);

    let usage = serde_json::to_string(&AgentEvent::Usage {
        usage: Usage {
            input_tokens: 6,
            output_tokens: 9,
        },
    })
    .unwrap();
    assert_eq!(
        usage,
        r#"{"type":"usage","usage":{"input_tokens":6,"output_tokens":9}}"#
    );

    let calls = serde_json::to_string(&AgentEvent::ToolCalls {
        calls: vec![ToolCall {
            id: "call_1".into(),
            name: "speak".into(),
            arguments: "{}".into(),
        }],
    })
    .unwrap();
    assert_eq!(
        calls,
        r#"{"type":"tool_calls","calls":[{"id":"call_1","name":"speak","arguments":"{}"}]}"#
    );
}

#[test]
fn stop_reason_mapping() {
    assert_eq!(StopReason::from_protocol("stop"), StopReason::EndTurn);
    assert_eq!(StopReason::from_protocol("end_turn"), StopReason::EndTurn);
    assert_eq!(StopReason::from_protocol("length"), StopReason::MaxTokens);
    assert_eq!(
        StopReason::from_protocol("max_tokens"),
        StopReason::MaxTokens
    );
    assert_eq!(
        StopReason::from_protocol("tool_calls"),
        StopReason::ToolCalls
    );
    assert_eq!(StopReason::from_protocol("tool_use"), StopReason::ToolCalls);
    assert_eq!(
        StopReason::from_protocol("stop_sequence"),
        StopReason::Other
    );
    assert_eq!(StopReason::from_protocol("未知新原因"), StopReason::Other);

    // from/to_protocol 互逆（provider 多轮判断用 to_protocol）
    for raw in ["end_turn", "max_tokens", "tool_calls"] {
        assert_eq!(StopReason::from_protocol(raw).to_protocol(), raw);
    }
}
