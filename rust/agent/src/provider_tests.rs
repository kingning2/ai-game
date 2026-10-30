//! provider.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

#[test]
fn build_provider_selects_by_kind() {
    let config = AgentConfig {
        provider: ProviderKind::Anthropic,
        ..Default::default()
    };
    assert_eq!(build_provider(&config).unwrap().id(), "anthropic");

    let config = AgentConfig {
        provider: ProviderKind::OpenAi,
        ..Default::default()
    };
    assert_eq!(build_provider(&config).unwrap().id(), "openai");
}

/// 未配 Key 也能构建（调用时才报 Config，且在发请求前就报）
#[tokio::test]
async fn build_without_key_is_ok() {
    let config = AgentConfig::default();
    let provider = build_provider(&config).unwrap();
    let request = ChatRequest {
        messages: vec![crate::ChatMessage::user("hi")],
        ..ChatRequest::default()
    };
    assert!(matches!(
        provider.chat_stream(request).await,
        Err(AgentError::Config(_))
    ));
}
