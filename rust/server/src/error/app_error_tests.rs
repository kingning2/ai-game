//! app_error.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use std::io;

/// code 段与 status 必须同向：4xx ↔ 1xxxx，5xx ↔ 5xxxx
#[test]
fn code_segment_matches_status_class() {
    let cases: Vec<AppError> = vec![
        AppError::BadRequest("x".into()),
        AppError::Unauthorized,
        AppError::Forbidden,
        AppError::NotFound("x".into()),
        AppError::Conflict("x".into()),
        AppError::MethodNotAllowed,
        AppError::PayloadTooLarge,
        AppError::UnsupportedMediaType,
        AppError::Unprocessable("x".into()),
        AppError::TooManyRequests,
        AppError::Timeout,
        AppError::internal(io::Error::other("boom")),
        AppError::ServiceUnavailable,
    ];
    for err in cases {
        let status = err.status();
        let code = err.code();
        assert!(!err.to_string().is_empty());
        if status.is_client_error() {
            assert!((10001..=10099).contains(&code), "{err:?} code={code}");
        } else {
            assert!((50000..=59999).contains(&code), "{err:?} code={code}");
        }
    }
}

#[test]
fn conversions() {
    assert!(matches!(
        AppError::from(io::Error::other("disk")),
        AppError::Internal(_)
    ));
    // Internal 的原因通过 source 链可取回，但 Display 不泄露
    let err = AppError::internal(io::Error::other("db password leaked"));
    assert_eq!(err.to_string(), "内部服务错误");
    assert!(std::error::Error::source(&err).is_some());
}

/// Elapsed 无公开构造器，用真实超时产生后验证 From 转换
#[tokio::test]
async fn elapsed_maps_to_timeout() {
    let elapsed = tokio::time::timeout(std::time::Duration::ZERO, std::future::pending::<()>())
        .await
        .unwrap_err();
    assert!(matches!(AppError::from(elapsed), AppError::Timeout));
}

/// 映射原则：客户端输入 4xx，服务端/上游 5xx，完整上下文只进 source 链
#[test]
fn agent_error_mapping_semantics() {
    // 客户端输入问题 → 400，文案可见
    let err = AppError::from(AgentError::invalid_messages());
    assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    assert!(err.to_string().contains("user/assistant"));

    // 上游 429 → 限流；上游 5xx → 上游不可用
    let err = AppError::from(AgentError::upstream(StatusCode::TOO_MANY_REQUESTS, "qps"));
    assert_eq!(err.status(), StatusCode::TOO_MANY_REQUESTS);
    let err = AppError::from(AgentError::upstream(StatusCode::BAD_GATEWAY, "boom"));
    assert_eq!(err.status(), StatusCode::SERVICE_UNAVAILABLE);

    // 上游 4xx（我们的凭证/请求形状问题）→ Internal，不透传上游 body
    let err = AppError::from(AgentError::upstream(
        StatusCode::UNAUTHORIZED,
        "secret detail",
    ));
    assert_eq!(err.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(err.to_string(), "内部服务错误");
    assert!(std::error::Error::source(&err).is_some());

    // 网络层失败 → 上游不可用
    let net: BoxError = std::io::Error::other("connection reset").into();
    let err = AppError::from(AgentError::Stream(net));
    assert_eq!(err.status(), StatusCode::SERVICE_UNAVAILABLE);

    // 协议异常 → Internal（依赖行为异常，值得 error 级日志）
    let err = AppError::from(AgentError::Protocol("unknown frame".into()));
    assert_eq!(err.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
