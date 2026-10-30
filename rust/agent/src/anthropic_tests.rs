//! `anthropic.rs` 的单元测试。

use super::*;
use crate::message::{ToolCall, ToolResult};
use crate::stream::ToolCallDraft;

fn decode(state: &mut StreamState, data: &str) -> Vec<AgentEvent> {
    decode_frame(state, data).unwrap()
}

#[test]
fn full_stream_synthesis() {
    let mut state = StreamState::default();
    // message_start 只记 input，不产事件
    assert!(decode(
        &mut state,
        r#"{"type":"message_start","message":{"id":"msg_1","usage":{"input_tokens":25,"output_tokens":0}}}"#
    ).is_empty());

    assert_eq!(
        decode(
            &mut state,
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"你好"}}"#
        ),
        [AgentEvent::TextDelta {
            delta: "你好".into()
        }]
    );
    assert_eq!(
        decode(
            &mut state,
            r#"{"type":"content_block_delta","index":1,"delta":{"type":"thinking_delta","thinking":"推理中"}}"#
        ),
        [AgentEvent::ThinkingDelta {
            delta: "推理中".into()
        }]
    );

    // message_delta 合成 Usage（input 来自 message_start 的记忆）
    assert_eq!(
        decode(
            &mut state,
            r#"{"type":"message_delta","delta":{"stop_reason":"max_tokens","stop_sequence":null},"usage":{"output_tokens":15}}"#
        ),
        [AgentEvent::Usage {
            usage: Usage {
                input_tokens: 25,
                output_tokens: 15
            }
        }]
    );

    // message_stop 合成 Done，用上一步记下的 stop_reason
    assert_eq!(
        decode(&mut state, r#"{"type":"message_stop"}"#),
        [AgentEvent::Done {
            stop_reason: StopReason::MaxTokens
        }]
    );
}

/// tool_use 全流程：start 带 id/name → input_json_delta 拼参数 →
/// message_delta 记 stop_reason=tool_use → message_stop 合成 ToolCalls + Done
#[test]
fn tool_use_stream_aggregates() {
    let mut state = StreamState::default();
    decode(
        &mut state,
        r#"{"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"toolu_1","name":"speak"}}"#,
    );
    decode(
        &mut state,
        r#"{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"{\"text\":"}}"#,
    );
    decode(
        &mut state,
        r#"{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"\"发言\"}"}}"#,
    );

    assert_eq!(
        state.tool_calls,
        [ToolCallDraft {
            id: "toolu_1".into(),
            name: "speak".into(),
            arguments: r#"{"text":"发言"}"#.into(),
        }]
    );

    decode(
        &mut state,
        r#"{"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":42}}"#,
    );
    let events = decode(&mut state, r#"{"type":"message_stop"}"#);
    assert_eq!(
        events,
        [
            AgentEvent::ToolCalls {
                calls: vec![ToolCall {
                    id: "toolu_1".into(),
                    name: "speak".into(),
                    arguments: r#"{"text":"发言"}"#.into(),
                }]
            },
            AgentEvent::Done {
                stop_reason: StopReason::ToolCalls
            }
        ]
    );
}

#[test]
fn ping_and_unknown_types_ignored() {
    let mut state = StreamState::default();
    assert!(decode(&mut state, r#"{"type":"ping"}"#).is_empty());
    assert!(
        decode(
            &mut state,
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text"}}"#
        )
        .is_empty()
    );
    assert!(decode(&mut state, r#"{"type":"brand_new_frame_v99"}"#).is_empty());
}

#[test]
fn error_frame_is_err() {
    let mut state = StreamState::default();
    let err = decode_frame(
        &mut state,
        r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#,
    )
    .unwrap_err();
    assert!(matches!(err, AgentError::UpstreamEvent(message) if message == "Overloaded"));
}

/// 裸 error JSON（无 type 字段）也能兜住
#[test]
fn bare_error_object_is_err() {
    let mut state = StreamState::default();
    let err = decode_frame(&mut state, r#"{"error":{"message":"boom"}}"#).unwrap_err();
    assert!(matches!(err, AgentError::UpstreamEvent(message) if message == "boom"));
}

#[test]
fn malformed_frame_is_protocol_error() {
    let mut state = StreamState::default();
    let err = decode_frame(&mut state, "{{{").unwrap_err();
    assert!(matches!(err, AgentError::Protocol(_)));
    assert!(err.to_string().contains("anthropic frame"));
}

// ===== split_messages =====

#[test]
fn system_extracted_and_adjacent_roles_merged() {
    let messages = vec![
        ChatMessage::system("规则一"),
        ChatMessage::system("规则二"),
        ChatMessage::user("第一问"),
        ChatMessage::user("补充"),
        ChatMessage::assistant("回答"),
        ChatMessage::user("第二问"),
    ];
    let (system, turns) = split_messages(&messages);
    assert_eq!(system.as_deref(), Some("规则一\n\n规则二"));
    assert_eq!(turns.len(), 3);
    assert_eq!(turns[0]["role"], "user");
    // 相邻同 role 合并为同一 turn 的多个 text block
    assert_eq!(turns[0]["content"][0]["text"], "第一问");
    assert_eq!(turns[0]["content"][1]["text"], "补充");
    assert_eq!(turns[1]["role"], "assistant");
    assert_eq!(turns[2]["content"][0]["text"], "第二问");
}

#[test]
fn no_system_gives_none() {
    let messages = vec![ChatMessage::user("hi")];
    let (system, turns) = split_messages(&messages);
    assert_eq!(system, None);
    assert_eq!(turns.len(), 1);
}

/// 多轮工具往返：assistant 调用变 tool_use block，结果并入后续 user turn
#[test]
fn tool_roundtrip_becomes_blocks() {
    let messages = vec![
        ChatMessage::system("规则"),
        ChatMessage::user("轮到你"),
        ChatMessage::assistant_with_tools(
            "让我想想",
            vec![ToolCall {
                id: "toolu_1".into(),
                name: "speak".into(),
                arguments: r#"{"text":"大家好"}"#.into(),
            }],
        ),
        ChatMessage::tool_result("toolu_1", "已记录"),
        ChatMessage::user("继续"),
    ];
    let (system, turns) = split_messages(&messages);
    assert_eq!(system.as_deref(), Some("规则"));
    assert_eq!(turns.len(), 3, "tool_result 并入 user，不单独成 turn");

    // assistant turn：text block + tool_use block
    let assistant = &turns[1];
    assert_eq!(assistant["role"], "assistant");
    assert_eq!(assistant["content"][0]["type"], "text");
    assert_eq!(assistant["content"][0]["text"], "让我想想");
    assert_eq!(assistant["content"][1]["type"], "tool_use");
    assert_eq!(assistant["content"][1]["id"], "toolu_1");
    assert_eq!(assistant["content"][1]["name"], "speak");
    assert_eq!(assistant["content"][1]["input"]["text"], "大家好");

    // 结果 turn：tool_result block 挂在 user role 里
    let result = &turns[2];
    assert_eq!(result["role"], "user");
    assert_eq!(result["content"][0]["type"], "tool_result");
    assert_eq!(result["content"][0]["tool_use_id"], "toolu_1");
    assert_eq!(result["content"][0]["content"], "已记录");
    assert_eq!(result["content"][1]["text"], "继续");
}

/// arguments 解析失败时保 _raw 不丢信息
#[test]
fn broken_arguments_fallback_to_raw() {
    let messages = vec![ChatMessage::assistant_with_tools(
        "",
        vec![ToolCall {
            id: "t1".into(),
            name: "vote".into(),
            arguments: "不是json".into(),
        }],
    )];
    let (_, turns) = split_messages(&messages);
    let block = &turns[0]["content"][0];
    assert_eq!(block["type"], "tool_use");
    assert_eq!(block["input"]["_raw"], "不是json");
}

/// 结果缺失前置 assistant 调用时也不丢：兜底开新 user turn
#[test]
fn orphan_tool_result_keeps_own_turn() {
    let messages = vec![ChatMessage::tool_result("t1", "结果")];
    let (_, turns) = split_messages(&messages);
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0]["role"], "user");
    assert_eq!(turns[0]["content"][0]["type"], "tool_result");
}

/// 空文本 + 只有 tool_calls 的 assistant 消息不产生空 text block
#[test]
fn empty_assistant_text_skipped() {
    let messages = vec![ChatMessage::assistant_with_tools(
        "",
        vec![ToolCall {
            id: "t1".into(),
            name: "speak".into(),
            arguments: "{}".into(),
        }],
    )];
    let (_, turns) = split_messages(&messages);
    let content = turns[0]["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "tool_use");
}

#[test]
fn tool_result_struct_carries_call_id() {
    let result = ToolResult {
        call_id: "t9".into(),
        content: "ok".into(),
    };
    assert_eq!(result.call_id, "t9");
}
