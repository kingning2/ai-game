//! mod.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

#[test]
fn camps_and_gods_are_consistent() {
    assert_eq!(Role::Werewolf.camp(), Camp::Werewolf);
    for role in [
        Role::Seer,
        Role::Witch,
        Role::Guard,
        Role::Hunter,
        Role::Idiot,
        Role::Knight,
        Role::Villager,
    ] {
        assert_eq!(role.camp(), Camp::Good, "{role:?} 应属好人阵营");
        assert_eq!(
            role.is_god(),
            role != Role::Villager,
            "{role:?} 神职判定错误"
        );
    }
}

#[test]
fn serde_uses_lowercase() {
    assert_eq!(
        serde_json::to_string(&Role::Werewolf).unwrap(),
        "\"werewolf\""
    );
    assert_eq!(
        serde_json::from_str::<Role>("\"seer\"").unwrap(),
        Role::Seer
    );
}
