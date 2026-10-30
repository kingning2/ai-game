//! memory.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use werewolf::action::Action;
use werewolf::board::BoardConfig;
use werewolf::phase::NightStage;

/// 双狼 all_stars 局推进到白天发言一圈后返回引擎（含夜聊/天亮/发言事实）
fn two_wolf_game_with_speech() -> GameEngine {
    let mut board = BoardConfig::all_stars();
    board.werewolves = 2;
    let mut engine = GameEngine::new(board, 79).expect("合法板子");
    // 守卫空守
    let (actor, _) = engine.pending().expect("守卫窗口");
    engine
        .apply(actor, Action::GuardProtect { target: None })
        .expect("空守合法");
    // 狼聊：两只狼各沉默一条 → 刀 3 号
    while matches!(
        engine.phase(),
        werewolf::phase::Phase::Night {
            stage: NightStage::WerewolfChat,
            ..
        }
    ) {
        let (actor, _) = engine.pending().expect("狼聊窗口");
        engine
            .apply(actor, Action::WolfChat { text: None })
            .expect("沉默合法");
    }
    let (actor, _) = engine.pending().expect("刀人窗口");
    engine
        .apply(actor, Action::WerewolfKill { target: 3 })
        .expect("刀人合法");
    // 女巫不用药、预言家验 4 号（狼聊后剩余窗口只有这两个角色）
    while matches!(engine.phase(), werewolf::phase::Phase::Night { .. }) {
        let (actor, _) = engine.pending().expect("夜晚窗口");
        let action = match engine.players()[actor as usize].role {
            werewolf::role::Role::Witch => Action::WitchAct {
                save: false,
                poison: None,
            },
            werewolf::role::Role::Seer => Action::SeerCheck { target: 4 },
            role => unreachable!("剩余夜晚窗口只有女巫/预言家，遇到 {role:?}"),
        };
        engine.apply(actor, action).expect("夜晚动作合法");
    }
    // 白天发言一圈
    while matches!(engine.phase(), werewolf::phase::Phase::DaySpeech { .. }) {
        let (actor, _) = engine.pending().expect("发言窗口");
        engine
            .apply(
                actor,
                Action::Speech {
                    text: "我是好人".into(),
                },
            )
            .expect("发言合法");
    }
    engine
}

#[test]
fn villager_memory_has_no_wolf_chat_or_checks() {
    let engine = two_wolf_game_with_speech();
    let villager = engine
        .players()
        .iter()
        .find(|p| p.role == Role::Villager && p.alive)
        .map(|p| p.id)
        .expect("必有活民");
    let memory = recall(&engine, villager).expect("合法视角");
    assert!(
        memory.entries.iter().all(|entry| entry.tag != "wolfchat"),
        "民不得记得狼聊"
    );
    assert!(
        memory.entries.iter().all(|entry| entry.tag != "seercheck"),
        "民不得记得查验"
    );
    assert!(
        memory.entries.iter().any(|entry| entry.tag == "speech"),
        "应记得公开发言"
    );
    assert!(
        memory.entries.iter().any(|entry| entry.tag == "dawn"),
        "应记得天亮播报"
    );
}

#[test]
fn wolf_memory_keeps_council() {
    let engine = two_wolf_game_with_speech();
    let wolf = engine
        .players()
        .iter()
        .find(|p| p.role == Role::Werewolf && p.alive)
        .map(|p| p.id)
        .expect("必有活狼");
    let memory = recall(&engine, wolf).expect("合法视角");
    assert_eq!(
        memory
            .entries
            .iter()
            .filter(|entry| entry.tag == "wolfchat")
            .count(),
        2,
        "狼应记得两条耳语（含沉默）"
    );
}

#[test]
fn seer_memory_keeps_private_checks() {
    let engine = two_wolf_game_with_speech();
    let seer = engine
        .players()
        .iter()
        .find(|p| p.role == Role::Seer)
        .map(|p| p.id)
        .expect("必有预言家");
    let memory = recall(&engine, seer).expect("合法视角");
    let checks: Vec<_> = memory
        .entries
        .iter()
        .filter(|entry| entry.tag == "seercheck")
        .collect();
    assert_eq!(checks.len(), 1, "预言家应记得自己验过 4 号");
    assert!(checks[0].text.contains("4 号"), "记录应含目标座位");
    assert!(
        memory.render().contains("[#1·seercheck]"),
        "render 应带查验标签"
    );
}

#[test]
fn memory_is_deterministic_and_ordered() {
    let engine = two_wolf_game_with_speech();
    let any = 0;
    let a = recall(&engine, any).expect("合法视角");
    let b = recall(&engine, any).expect("合法视角");
    assert_eq!(a, b, "同一状态同一玩家记忆必须一致");
    assert!(
        a.entries.windows(2).all(|w| w[0].seq <= w[1].seq),
        "记忆必须按时间片序"
    );
}
