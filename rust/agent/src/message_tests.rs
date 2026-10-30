//! `message.rs` 的单元测试。

use super::*;

/// serde 小写约定：server 的 DTO 直接复用本类型，字段值必须与各家协议一致
#[test]
fn role_serde_lowercase() {
    assert_eq!(serde_json::to_string(&Role::User).unwrap(), "\"user\"");
    let role: Role = serde_json::from_str("\"assistant\"").unwrap();
    assert_eq!(role, Role::Assistant);
}

#[test]
fn constructors_set_roles() {
    assert_eq!(ChatMessage::system("s").role, Role::System);
    assert_eq!(ChatMessage::user("u").role, Role::User);
    assert_eq!(ChatMessage::assistant("a").role, Role::Assistant);
}

#[test]
fn validate_rules() {
    let ok = ChatRequest {
        messages: vec![ChatMessage::user("hi")],
        ..ChatRequest::default()
    };
    assert!(ok.validate().is_ok());

    let empty = ChatRequest::default();
    assert!(matches!(
        empty.validate(),
        Err(AgentError::InvalidRequest(_))
    ));

    let only_system = ChatRequest {
        messages: vec![ChatMessage::system("你是助手")],
        ..ChatRequest::default()
    };
    assert!(matches!(
        only_system.validate(),
        Err(AgentError::InvalidRequest(_))
    ));
}

/// 多轮工具往返的组装：assistant 带调用 → tool 回结果，字段各归各位
#[test]
fn tool_roundtrip_messages() {
    let call = ToolCall {
        id: "call_1".into(),
        name: "speak".into(),
        arguments: r#"{"text":"大家好"}"#.into(),
    };
    let asked = ChatMessage::assistant_with_tools("", vec![call.clone()]);
    assert_eq!(asked.role, Role::Assistant);
    assert_eq!(asked.tool_calls, vec![call.clone()]);
    assert!(asked.tool_results.is_empty());

    let answered = ChatMessage::tool_result("call_1", "ok");
    assert_eq!(answered.role, Role::Tool);
    assert_eq!(answered.tool_results[0].call_id, "call_1");
    assert_eq!(answered.tool_results[0].content, "ok");
    assert!(answered.tool_calls.is_empty());
    assert!(answered.tool_results.iter().all(|r| r.call_id == call.id));
}

/// 带 tools 的请求也能过校验；空 tools 与 ToolChoice::Auto 是默认值
#[test]
fn request_with_tools_validates() {
    let request = ChatRequest {
        messages: vec![ChatMessage::user("发言")],
        tools: vec![ToolDefinition::new(
            "speak",
            "提交发言",
            serde_json::json!({"type": "object"}),
        )],
        tool_choice: ToolChoice::Required,
        ..ChatRequest::default()
    };
    assert!(request.validate().is_ok());
    assert_eq!(request.tools.len(), 1);
    assert_eq!(request.tool_choice, ToolChoice::Required);
}
