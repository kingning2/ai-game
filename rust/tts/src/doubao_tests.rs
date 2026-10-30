//! doubao.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

#[test]
fn device_ids_in_python_range_and_distinct() {
    let (a, b) = (device_id(), device_id());
    for id in [a, b] {
        assert!(
            (7_400_000_000_000_000_000..7_500_000_000_000_000_000).contains(&id),
            "id {id} 超出 Python randint 区间"
        );
    }
    assert_ne!(a, b, "连续两次取值应不同（概率性，但 1e17 空间内必异）");
}

#[test]
fn url_carries_protocol_params() {
    let config = DoubaoConfig::new("sessionid=abc");
    let request = build_request(&config).expect("构造");
    let url = request.uri().to_string();
    assert!(url.starts_with(WS_URL), "{url}");
    assert!(url.contains("speaker=zh_female_taozi_conversation_v4_wvae_bigtts"));
    assert!(url.contains("format=aac"));
    assert!(url.contains("aid=497858"));
    assert!(url.contains("real_aid=497858"));
    assert!(url.contains("device_platform=web"));
    assert!(url.contains("speech_rate=0"));
    assert!(url.contains("web_tab_id="));
    assert_eq!(
        request.headers().get(ORIGIN).unwrap(),
        "https://www.doubao.com"
    );
    assert_eq!(request.headers().get(COOKIE).unwrap(), "sessionid=abc");
}

#[test]
fn speaker_resolution() {
    assert_eq!(resolve_speaker("rap"), "zh_male_rap_mars_bigtts");
    assert_eq!(
        resolve_speaker("zh_female_taozi_conversation_v4_wvae_bigtts"),
        "zh_female_taozi_conversation_v4_wvae_bigtts"
    );
    assert_eq!(resolve_speaker(""), DEFAULT_SPEAKER);
    assert_eq!(resolve_speaker("未知"), DEFAULT_SPEAKER);
}

#[tokio::test]
async fn rejects_missing_cookie_before_connecting() {
    let config = DoubaoConfig::new("");
    let err = synthesize(&config, "你好").await.expect_err("应拒绝");
    assert!(matches!(err, TtsError::InvalidInput(_)), "{err:?}");
}

#[tokio::test]
async fn rejects_empty_text_before_connecting() {
    let config = DoubaoConfig::new("sessionid=abc");
    let err = synthesize(&config, "   ").await.expect_err("应拒绝");
    assert!(matches!(err, TtsError::InvalidInput(_)), "{err:?}");
}

#[tokio::test]
async fn rejects_mp3_before_connecting() {
    // 豆包云端不支持 mp3，客户端直接拦下（不发起连接撞风控）
    let mut config = DoubaoConfig::new("sessionid=abc");
    config.format = "mp3".to_string();
    let err = synthesize(&config, "你好").await.expect_err("应拒绝");
    let TtsError::InvalidInput(msg) = &err else {
        panic!("应返回参数错误，实际 {err:?}");
    };
    assert!(msg.contains("mp3"), "{msg}");
}

#[test]
fn describes_risk_control_errors() {
    assert!(describe_doubao_error("block").contains("风控"));
    assert!(describe_doubao_error("系统错误").contains("重试"));
    assert_eq!(describe_doubao_error("其他"), "其他");
}
