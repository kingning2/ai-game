//! board.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

#[test]
fn presets_match_expected_totals() {
    assert_eq!(BoardConfig::nine_standard().total(), 9);
    assert_eq!(BoardConfig::all_stars().total(), 12);
    assert!(BoardConfig::nine_standard().validate().is_ok());
    assert!(BoardConfig::all_stars().validate().is_ok());
}

#[test]
fn roles_expand_to_total() {
    let board = BoardConfig::all_stars();
    let roles = board.roles();
    assert_eq!(roles.len(), board.total() as usize);
    let mut counts = BTreeMap::new();
    for role in roles {
        *counts.entry(role).or_insert(0) += 1;
    }
    assert_eq!(counts[&Role::Werewolf], 4);
    assert_eq!(counts.get(&Role::Knight), None);
    assert_eq!(counts.get(&Role::Idiot), None);
}

#[test]
fn validate_rejects_broken_boards() {
    let mut board = BoardConfig::nine_standard();
    board.werewolves = 0;
    assert_eq!(board.validate(), Err(BoardError::NoWerewolf));

    let mut board = BoardConfig::nine_standard();
    board.werewolves = 9;
    board.villagers = 0;
    board.seer = 0;
    board.witch = 0;
    board.hunter = 0;
    assert_eq!(board.validate(), Err(BoardError::NoGood));

    let mut board = BoardConfig::nine_standard();
    board.witch = 2;
    assert!(matches!(
        board.validate(),
        Err(BoardError::DuplicateGod(name)) if name == "witch"
    ));
}

#[test]
fn serde_round_trip() {
    let board = BoardConfig::all_stars();
    let json = serde_json::to_string(&board).unwrap();
    assert_eq!(serde_json::from_str::<BoardConfig>(&json).unwrap(), board);
}
