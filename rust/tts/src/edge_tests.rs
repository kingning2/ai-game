//! edge.rs 的单元测试（事故丢失后重写）。
//!
//! 合成函数的联网路径不在此测（集成层面验证）；这里锁定：
//! 语速换算、输入校验（离线可测，先于网络）、音色分配的确定性与约束。

use super::*;
use crate::TtsError;

#[test]
fn rate_of_maps_speed_to_ssml() {
    assert_eq!(rate_of(1.0), "+0%");
    assert_eq!(rate_of(1.15), "+15%");
    assert_eq!(rate_of(0.5), "-50%");
    assert_eq!(rate_of(2.0), "+100%");
    assert_eq!(rate_of(1.149), "+15%"); // 四舍五入
}

#[test]
fn voices_pool_invariants() {
    assert_eq!(
        VOICES.len(),
        6,
        "Edge 普通话音色实测共 6 个，变动需重新实测"
    );
    let mut sorted = VOICES.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 6, "音色池有重复");
    assert!(VOICES.contains(&HOST_VOICE), "主持人音色必须在池内");
}

#[test]
fn assign_never_uses_host_voice() {
    for seed in [0u64, 42, u64::MAX] {
        for n in 1..=12 {
            for voice in assign(seed, n) {
                assert_ne!(voice, HOST_VOICE, "选手分到了主持人音色");
            }
        }
    }
}

#[test]
fn assign_is_deterministic_per_game() {
    assert_eq!(assign(7, 6), assign(7, 6));
    // 首轮（n≤5）覆盖全部 5 个非主持音色，各不相同
    let first_round = assign(7, 5);
    let mut sorted = first_round.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 5, "首轮选手音色应互不相同");
    // 超池循环复用：第 6 位与第 1 位同音色
    assert_eq!(assign(7, 6)[5], first_round[0]);
}

#[tokio::test]
async fn synthesize_rejects_unknown_voice_offline() {
    let err = synthesize("zh-CN-NotAVoice", "你好", 1.0)
        .await
        .unwrap_err();
    assert!(matches!(err, TtsError::InvalidInput(_)));
}

#[tokio::test]
async fn synthesize_rejects_out_of_range_speed_offline() {
    for speed in [0.4, 2.1, -1.0] {
        let err = synthesize(HOST_VOICE, "你好", speed).await.unwrap_err();
        assert!(
            matches!(err, TtsError::InvalidInput(_)),
            "语速 {speed} 应被拒"
        );
    }
}
