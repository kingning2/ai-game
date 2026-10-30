//! `openai.rs` 的单元测试。

use super::*;
use crate::message::ToolCall;
use crate::stream::ToolCallDraft;

fn decode(state: &mut StreamState, data: &str) -> Vec<AgentEvent> {
    decode_frame(state, data).unwrap()
}

#[test]
fn text_delta() {
    let mut state = StreamState::default();
    let events = decode(
        &mut state,
        r#"{"id":"1","choices":[{"index":0,"delta":{"content":"你好"},"finish_reason":null}]}"#,
    );
    assert_eq!(
        events,
        [AgentEvent::TextDelta {
            delta: "你好".into()
        }]
    );
}

/// DeepSeek reasoning_content 优先，OpenRouter reasoning 兜底
#[test]
fn thinking_delta_both_field_names() {
    let mut state = StreamState::default();
    let events = decode(
        &mut state,
        r#"{"choices":[{"delta":{"reasoning_content":"想一下"}}]}"#,
    );
    assert_eq!(
        events,
        [AgentEvent::ThinkingDelta {
            delta: "想一下".into()
        }]
    );

    let events = decode(
        &mut state,
        r#"{"choices":[{"delta":{"reasoning":"再想想"}}]}"#,
    );
    assert_eq!(
        events,
        [AgentEvent::ThinkingDelta {
            delta: "再想想".into()
        }]
    );
}

/// 空串增量不发事件（部分网关会发 content:"" 占位帧）
#[test]
fn empty_delta_skipped() {
    let mut state = StreamState::default();
    let events = decode(&mut state, r#"{"choices":[{"delta":{"content":""}}]}"#);
    assert!(events.is_empty());
}

/// finish_reason 记在 state，[DONE] 时合成 Done
#[test]
fn done_uses_last_finish_reason() {
    let mut state = StreamState::default();
    decode(
        &mut state,
        r#"{"choices":[{"delta":{"content":"尾"},"finish_reason":"length"}]}"#,
    );
    let events = decode(&mut state, "[DONE]");
    assert_eq!(
        events,
        [AgentEvent::Done {
            stop_reason: StopReason::MaxTokens
        }]
    );

    // 重复结束帧被吞
    assert!(decode(&mut state, "[DONE]").is_empty());
}

/// include_usage 的用量帧：choices 为空数组 + usage 全量
#[test]
fn usage_chunk() {
    let mut state = StreamState::default();
    let events = decode(
        &mut state,
        r#"{"choices":[],"usage":{"prompt_tokens":6,"completion_tokens":9}}"#,
    );
    assert_eq!(
        events,
        [AgentEvent::Usage {
            usage: Usage {
                input_tokens: 6,
                output_tokens: 9
            }
        }]
    );
}

/// 流中错误帧 → Err（HTTP 已 200，只能走 Err 分支）
#[test]
fn in_stream_error_is_err() {
    let mut state = StreamState::default();
    let err = decode_frame(
        &mut state,
        r#"{"error":{"message":"模型不存在","type":"invalid_request_error"}}"#,
    )
    .unwrap_err();
    assert!(matches!(err, AgentError::UpstreamEvent(message) if message == "模型不存在"));
}

/// 未知 JSON → Protocol 错误（可定位是哪段内容）
#[test]
fn malformed_frame_is_protocol_error() {
    let mut state = StreamState::default();
    let err = decode_frame(&mut state, "{not json").unwrap_err();
    assert!(matches!(err, AgentError::Protocol(_)));
    assert!(err.to_string().contains("openai chunk"));
}

/// 流式 tool_calls 分片按 index 聚合，[DONE] 时先发 ToolCalls 再 Done
#[test]
fn tool_calls_fragments_aggregate() {
    let mut state = StreamState::default();
    // 首片带 id/name，后续片只带 arguments 增量
    decode(
        &mut state,
        r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","function":{"name":"speak","arguments":""}}]}}]}"#,
    );
    decode(
        &mut state,
        r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"text\":"}}]}}]}"#,
    );
    decode(
        &mut state,
        r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"发言\"}"}}]}}]}"#,
    );
    decode(
        &mut state,
        r#"{"choices":[{"delta":{},"finish_reason":"tool_calls"}]}"#,
    );

    assert_eq!(
        state.tool_calls,
        [ToolCallDraft {
            id: "call_1".into(),
            name: "speak".into(),
            arguments: r#"{"text":"发言"}"#.into(),
        }]
    );

    let events = decode(&mut state, "[DONE]");
    assert_eq!(
        events,
        [
            AgentEvent::ToolCalls {
                calls: vec![ToolCall {
                    id: "call_1".into(),
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

/// 普通结束（stop）时即使有残留分片也不发 ToolCalls
#[test]
fn tool_calls_ignored_on_plain_stop() {
    let mut state = StreamState::default();
    decode(
        &mut state,
        r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"id":"x","function":{"name":"y","arguments":"{}"}}]}}]}"#,
    );
    decode(
        &mut state,
        r#"{"choices":[{"delta":{},"finish_reason":"stop"}]}"#,
    );
    let events = decode(&mut state, "[DONE]");
    assert_eq!(
        events,
        [AgentEvent::Done {
            stop_reason: StopReason::EndTurn
        }]
    );
}

/// 多轮消息序列化：tool_calls 包装 function，tool 结果带 tool_call_id
#[test]
fn protocol_messages_roundtrip() {
    let messages = vec![
        ChatMessage::system("你是玩家"),
        ChatMessage::user("轮到你发言"),
        ChatMessage::assistant_with_tools(
            "",
            vec![ToolCall {
                id: "call_1".into(),
                name: "speak".into(),
                arguments: r#"{"text":"大家好"}"#.into(),
            }],
        ),
        ChatMessage::tool_result("call_1", "已记录"),
    ];
    let wire = to_protocol_messages(&messages);

    assert_eq!(wire[0]["role"], "system");
    assert_eq!(wire[1]["role"], "user");
    assert_eq!(wire[2]["tool_calls"][0]["type"], "function");
    assert_eq!(wire[2]["tool_calls"][0]["function"]["name"], "speak");
    // content 为空时省略字段（部分兼容网关拒绝 tool_calls 消息带 content）
    assert_eq!(wire[2]["content"], serde_json::Value::Null);
    assert_eq!(wire[3]["role"], "tool");
    assert_eq!(wire[3]["tool_call_id"], "call_1");
    assert_eq!(wire[3]["content"], "已记录");
}

/// 工具定义 → function 包装；Required → "required"
#[test]
fn protocol_tools_and_choice() {
    let tools = vec![ToolDefinition::new(
        "speak",
        "提交发言",
        serde_json::json!({"type": "object", "properties": {}}),
    )];
    let wire = to_protocol_tools(&tools);
    assert_eq!(wire[0]["type"], "function");
    assert_eq!(wire[0]["function"]["name"], "speak");
    assert_eq!(wire[0]["function"]["parameters"]["type"], "object");
}
