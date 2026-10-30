//! game_state.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use werewolf::BoardError;

fn fresh_engine(seed: u64) -> GameEngine {
    GameEngine::new(BoardConfig::nine_standard(), seed)
        .map_err(|err: BoardError| panic!("预设板子应恒合法：{err}"))
        .expect("预设板子应恒合法")
}

fn start_game(state: &GameState, game_id: i64, seed: u64) -> Result<GameView, InvalidTransition> {
    state.start(
        game_id,
        fresh_engine(seed),
        "nine_standard".into(),
        seed,
        vec!["选手".to_string(); 9],
        Vec::new(),
    )
}

#[test]
fn full_lifecycle() {
    let state = GameState::new();
    let view = start_game(&state, 7, 7).expect("新槽位可开局");
    assert_eq!(view.status, GameStatus::Running);
    assert_eq!(view.phase, "night");
    assert_eq!(view.board.as_deref(), Some("nine_standard"));
    assert_eq!(view.seed, 7);
    assert_eq!(state.latest(), Some(7));
    let summary = view.summary.expect("开局后应有总览");
    assert_eq!(summary.total, 9);
    assert_eq!(summary.alive_total, 9);

    state.pause(7).expect("running 可暂停");
    assert_eq!(state.view(7).expect("槽位存在").status, GameStatus::Paused);
    state.resume(7).expect("paused 可继续");
    assert_eq!(state.view(7).expect("槽位存在").status, GameStatus::Running);

    let view = state.end(7).expect("running 可结束");
    assert_eq!(view.status, GameStatus::Ended);
    assert!(view.outcome.is_none(), "手动结束未结算，outcome 应为空");
}

#[test]
fn invalid_transitions_rejected() {
    let state = GameState::new();

    let err = state.pause(1).expect_err("槽位不存在应 404 语义");
    assert_eq!(err, GameError::Config("对局不存在或已失效".into()));

    start_game(&state, 1, 1).expect("开局");
    let err = state
        .start(
            1,
            fresh_engine(2),
            "nine_standard".into(),
            2,
            vec!["选手".into(); 9],
            Vec::new(),
        )
        .expect_err("running 不能重复开局");
    assert_eq!(err.from, GameStatus::Running);

    state.pause(1).expect("暂停");
    let err = state.pause(1).expect_err("paused 不能再暂停");
    assert_eq!(
        err,
        GameError::Transition(InvalidTransition {
            action: "暂停",
            from: GameStatus::Paused
        })
    );
    let err = state
        .start(
            1,
            fresh_engine(2),
            "nine_standard".into(),
            2,
            vec!["选手".into(); 9],
            Vec::new(),
        )
        .expect_err("paused 不能开局");
    assert_eq!(err.from, GameStatus::Paused);

    let view = state.end(1).expect("paused 可结束");
    assert_eq!(view.status, GameStatus::Ended);
    let err = state.end(1).expect_err("ended 不能再结束");
    assert_eq!(
        err,
        GameError::Transition(InvalidTransition {
            action: "结束",
            from: GameStatus::Ended
        })
    );
}

#[test]
fn restart_after_end() {
    let state = GameState::new();
    start_game(&state, 3, 3).expect("第一局");
    state.end(3).expect("结束");
    assert_eq!(state.view(3).expect("槽位存在").status, GameStatus::Ended);

    start_game(&state, 3, 4).expect("ended 可重新开局");
    let view = state.view(3).expect("槽位存在");
    assert_eq!(view.status, GameStatus::Running);
    assert_eq!(view.seed, 4, "新局覆盖旧局元数据");
    assert_eq!(view.summary.expect("新局总览").alive_total, 9);
}

#[test]
fn multi_slots_are_isolated() {
    let state = GameState::new();
    start_game(&state, 1, 1).expect("房 A 开局");
    start_game(&state, 2, 2).expect("房 B 开局");
    assert_eq!(state.latest(), Some(2), "latest 记录最近启动的局");

    // 暂停 A 不影响 B
    state.pause(1).expect("暂停 A");
    assert_eq!(state.view(1).expect("A 存在").status, GameStatus::Paused);
    assert_eq!(state.view(2).expect("B 存在").status, GameStatus::Running);

    // 各自引擎互不串局（种子不同，局面天数推进独立可辨）
    state.end(2).expect("结束 B");
    assert_eq!(state.view(1).expect("A 存在").status, GameStatus::Paused);
    assert_eq!(state.view(2).expect("B 存在").status, GameStatus::Ended);
}

#[test]
fn fresh_ids_are_negative_and_unique() {
    let state = GameState::new();
    let a = state.fresh_id();
    let b = state.fresh_id();
    assert!(a < 0 && b < 0, "裸局 id 用负数，不与 db 正数 id 冲突");
    assert_ne!(a, b);
}

#[test]
fn memories_are_per_slot() {
    let state = GameState::new();
    start_game(&state, 1, 1).expect("局 1");
    start_game(&state, 2, 2).expect("局 2");
    state
        .with_memories(1, |memories| {
            memories.insert(0, AgentMemory::new("甲".to_string()));
        })
        .expect("局 1 写记忆");
    // 局 2 的记忆表不受局 1 影响
    let leaked = state
        .with_memories(2, |memories| memories.contains_key(&0))
        .expect("局 2 存在");
    assert!(!leaked, "记忆按槽位隔离");
}

#[test]
fn resolve_board_unknown_name() {
    assert!(GameState::resolve_board("nine_standard").is_some());
    assert!(GameState::resolve_board("all_stars").is_some());
    assert!(GameState::resolve_board("newbie_six").is_some());
    assert!(GameState::resolve_board("nope").is_none());
}
