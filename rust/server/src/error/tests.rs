//! mod.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

/// fallback 的语义正确性
#[tokio::test]
async fn fallback_semantics() {
    assert_eq!(
        fallback_404().await.0.status(),
        axum::http::StatusCode::NOT_FOUND
    );
    assert_eq!(
        fallback_405().await.0.status(),
        axum::http::StatusCode::METHOD_NOT_ALLOWED
    );
}
