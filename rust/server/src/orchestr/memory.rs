//! 对局记忆编排：把 game 引擎事件流映射为通用 LLM 记忆条目。
//!
//! game 只给事件流与玩家状态（基础能力），memory 只给通用记忆模型与渲染
//! （基础能力）；「对局里什么算一条记忆、谁看得见、标签是什么」是狼人杀
//! 领域语义，由本模块（server 编排层，同时依赖两包）完成映射：
//!
//! - `seq` = 对局天（时间片）；`tag` = `speech`/`wolfchat`/`dawn`/`vote`/`seercheck`
//! - 可见性规则与 [`GameSnapshot`](werewolf::snapshot::GameSnapshot) 严格一致
//!
//! 纯逻辑、零 IO、确定性（同一引擎状态 + 同一玩家 ⇒ 同一记忆）。

use agent::ChatMessage;
use memory::{AgentMemory, CompactionPlan, MemoryEntry};
use werewolf::GameError;
use werewolf::engine::GameEngine;
use werewolf::event::GameEvent;
use werewolf::phase::{CheckResult, PlayerId};
use werewolf::role::Role;

/// 从引擎事件流抽取 `player` 视角可见的对局事实（含预言家私有查验），
/// 按天归位后返回通用记忆模型。
pub fn recall(engine: &GameEngine, player: PlayerId) -> Result<AgentMemory, GameError> {
    GameError::check_seat(player, engine.total_players())?;
    let me = &engine.players()[player as usize];
    let is_wolf = me.role == Role::Werewolf;
    let mut out = AgentMemory::new(player.to_string());

    for event in engine.events() {
        match event {
            GameEvent::SpeechMade { day, speaker, text } => {
                out.push(MemoryEntry::new(
                    u64::from(*day),
                    "speech",
                    Some(speaker.to_string()),
                    format!("{speaker} 号发言：「{text}」"),
                ));
            }
            GameEvent::WolfChatSpoke { day, speaker, text } => {
                if speaker.role == Role::Werewolf && is_wolf {
                    out.push(MemoryEntry::new(
                        u64::from(*day),
                        "wolfchat",
                        Some(speaker.id.to_string()),
                        format!("{} 号耳语：「{text}」", speaker.id),
                    ));
                }
            }
            GameEvent::WolfChatPassed { day, speaker } => {
                if speaker.role == Role::Werewolf && is_wolf {
                    out.push(MemoryEntry::new(
                        u64::from(*day),
                        "wolfchat",
                        Some(speaker.id.to_string()),
                        format!("{} 号在耳语中保持沉默", speaker.id),
                    ));
                }
            }
            GameEvent::Dawn { day, deaths } => {
                let text = if deaths.is_empty() {
                    "昨夜平安，无人出局".to_string()
                } else {
                    let list = deaths
                        .iter()
                        .map(|death| format!("{} 号", death.player))
                        .collect::<Vec<_>>()
                        .join("、");
                    format!("{list} 倒在了夜里")
                };
                out.push(MemoryEntry::new(u64::from(*day), "dawn", None, text));
            }
            GameEvent::VoteResolved {
                day, tie, exiled, ..
            } => {
                let text = match (tie, exiled) {
                    (true, _) => "投票平票，无人被放逐".to_string(),
                    (false, Some(player)) => format!(
                        "{} 号被放逐，翻牌是{}",
                        player.id,
                        player.role.display_name()
                    ),
                    (false, None) => "投票结束，无人被放逐".to_string(),
                };
                out.push(MemoryEntry::new(u64::from(*day), "vote", None, text));
            }
            // 夜晚动作细节、决斗、翻牌、开局等私密/上帝信息不进玩家记忆
            // （夜晚死者统一走 Dawn 播报；玩家死因由决策层按需查快照）
            _ => {}
        }
    }

    // 预言家：私有查验记录并入记忆流
    if me.role == Role::Seer {
        for check in me.checks() {
            out.push(MemoryEntry::new(
                u64::from(check.day),
                "seercheck",
                Some(player.to_string()),
                format!(
                    "你查验了 {} 号：{}",
                    check.target,
                    match check.result {
                        CheckResult::Good => "好人",
                        CheckResult::Werewolf => "狼人",
                    }
                ),
            ));
        }
    }

    out.sort_by_seq();
    Ok(out)
}

/// 组装记忆压缩的 LLM 提示词：system 定压缩规则，user 给待压缩老段原文。
///
/// 压缩是语义工作，机械截断会丢真话——交给 LLM。规则偏向决策需要：
/// 身份判断/怀疑及理由、承诺、查验、投票与出局、关键冲突。
pub fn compaction_messages(plan: &CompactionPlan) -> Vec<ChatMessage> {
    vec![
        ChatMessage::system(
            "你是狼人杀对局的记忆压缩助手。把给定的对局记忆压缩为一段不超过 200 字的摘要，\
             必须保留：对每个人身份的判断与怀疑及理由、重要承诺、查验结果、投票与出局、关键冲突。\
             直接输出摘要正文，不要任何解释或前缀。",
        ),
        ChatMessage::user(&plan.older_text),
    ]
}

#[cfg(test)]
mod tests {
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
}
