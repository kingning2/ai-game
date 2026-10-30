//! error.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

/// Display 要带上关键上下文；source 链的覆盖在 agent 包的 e2e mock 测试里
#[test]
fn display_contains_context() {
    let err = AgentError::Upstream {
        status: StatusCode::UNAUTHORIZED,
        body: "invalid api key".into(),
    };
    assert!(err.to_string().contains("401"));
    assert!(err.to_string().contains("invalid api key"));

    let err = AgentError::missing_key();
    assert!(err.to_string().contains("API Key"));
}

/// 截断策略：按字符截（不切坏多字节中文），且只在这一个地方发生
#[test]
fn upstream_truncates_body_to_500_chars() {
    let body = "中".repeat(800);
    let err = AgentError::upstream(StatusCode::BAD_GATEWAY, &body);
    let AgentError::Upstream { body, .. } = err else {
        panic!("应为 Upstream 变体");
    };
    assert_eq!(body.chars().count(), 500);
}

#[test]
fn missing_key_message_points_to_settings() {
    let err = AgentError::missing_key();
    assert!(matches!(err, AgentError::Config(message) if message.contains("未配置")));
}
