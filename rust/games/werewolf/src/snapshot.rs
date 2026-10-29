use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::engine::GameEngine;
use crate::error::GameError;
use crate::event::GameEvent;
use crate::phase::{CheckRecord, DeathCause, Outcome, Phase, PlayerId, Viewer, WerewolfReason};
use crate::player::{PlayerState, SnapshotPlayer};
use crate::role::{Camp, Role};

/// 快照日志条目（上帝视角为全量；玩家视角裁掉夜晚动作细节）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotLog {
    pub day: u32,
    pub text: String,
}

/// 当前等待的窗口（驱动调用方轮转喂动作 / 请求 LLM 决策）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingWindow {
    pub actor: PlayerId,
    pub kind: String,
}

/// 某一时刻的对局快照 —— 前端观战页 / 玩家客户端的唯一数据源。
///
/// 保密规则：
/// - 身份：上帝视角全知；玩家视角只看自己、狼队友、死者翻牌、白痴翻牌
/// - 日志：玩家视角裁掉夜晚动作细节（守护/刀口/用药/验人），只保留公共播报
///   （天黑天亮 / 死者公布 / 发言 / 决斗 / 投票 / 翻牌 / 胜负）；狼刀决定对全狼队可见
/// - 验人记录：只挂预言家本人视角（`checks` 非空即本人）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSnapshot {
    pub day: u32,
    pub phase: Phase,
    pub players: Vec<SnapshotPlayer>,
    pub logs: Vec<SnapshotLog>,
    pub pending: Option<PendingWindow>,
    /// 对局结束时非空
    pub outcome: Option<Outcome>,
    /// 预言家本人视角的验人记录（他人视角恒为空）
    pub checks: Vec<CheckRecord>,
}

/// 事件的可见性：夜晚动作细节只有上帝（与当事者/狼队）可见，其余全部公开。
/// `viewer_is_wolf`：玩家视角是否狼（狼队夜聊只对狼开放，上帝全见）
fn is_visible_event(event: &GameEvent, viewer: Viewer, viewer_is_wolf: bool) -> bool {
    match event {
        GameEvent::NightAction { actor, action, .. } => match viewer {
            Viewer::God => true,
            Viewer::Player(id) => {
                actor.id == id
                    || (matches!(action, Action::WerewolfKill { .. })
                        && actor.role == Role::Werewolf)
            }
        },
        GameEvent::WolfChatSpoke { speaker, .. } | GameEvent::WolfChatPassed { speaker, .. } => {
            matches!(viewer, Viewer::God) || (speaker.role == Role::Werewolf && viewer_is_wolf)
        }
        _ => true,
    }
}

/// 事件渲染为一行日志文本
fn render_log(event: &GameEvent) -> (u32, String) {
    let text = match event {
        GameEvent::GameStarted { players, board } => {
            let board = board
                .iter()
                .map(|(name, count)| format!("{name}×{count}"))
                .collect::<Vec<_>>()
                .join("、");
            format!("{players} 人对局开始（{board}）")
        }
        GameEvent::NightBegan { day } => format!("第 {day} 夜降临"),
        GameEvent::NightAction { actor, action, .. } => {
            format!(
                "{} 号（{}）{}",
                actor.id,
                actor.role.display_name(),
                action.describe()
            )
        }
        GameEvent::Dawn { day, deaths } => {
            if deaths.is_empty() {
                format!("第 {day} 天亮起：昨夜是平安夜")
            } else {
                let list = deaths
                    .iter()
                    .map(|death| format!("{} 号", death.player))
                    .collect::<Vec<_>>()
                    .join("、");
                format!("第 {day} 天亮起：{list} 倒在了血泊中")
            }
        }
        GameEvent::SpeechMade { speaker, text, .. } => format!("{speaker} 号发言：「{text}」"),
        GameEvent::WolfChatSpoke { speaker, text, .. } => {
            format!(
                "{} 号（{}）耳语：「{text}」",
                speaker.id,
                speaker.role.display_name()
            )
        }
        GameEvent::WolfChatPassed { speaker, .. } => {
            format!(
                "{} 号（{}）在耳语中保持沉默",
                speaker.id,
                speaker.role.display_name()
            )
        }
        GameEvent::KnightDuelResolved {
            knight,
            target,
            target_is_wolf,
            loser,
            ..
        } => {
            if *target_is_wolf {
                format!("骑士 {knight} 号决斗 {target} 号成功，对方是狼人！")
            } else {
                format!(
                    "骑士 {knight} 号决斗 {target} 号失败，{} 号含恨倒下",
                    loser.id
                )
            }
        }
        GameEvent::VoteResolved {
            counts,
            tie,
            exiled,
            ..
        } => {
            let counts = counts
                .iter()
                .map(|(seat, count)| format!("{seat} 号 {count} 票"))
                .collect::<Vec<_>>()
                .join("，");
            if *tie {
                format!("投票结束（{counts}）——平票，今日无人放逐")
            } else if let Some(player) = exiled {
                format!("投票结束（{counts}）——{} 号被放逐", player.id)
            } else {
                format!("投票结束（{counts}）——无人被放逐")
            }
        }
        GameEvent::IdiotRevealed { idiot, .. } => {
            format!("{idiot} 号亮出身份：是白痴！免于放逐但失去投票权")
        }
        GameEvent::HunterShotFired { hunter, target, .. } => match target {
            Some(target) => format!("猎人 {} 号开枪带走了 {} 号", hunter.id, target.id),
            None => format!("猎人 {} 号放弃开枪", hunter.id),
        },
        GameEvent::PlayerDied { player, cause, .. } => {
            let cause = match cause {
                DeathCause::Slain => "夜晚遇害",
                DeathCause::Poisoned => "中毒身亡",
                DeathCause::Exiled => "被投票放逐",
                DeathCause::Shot => "被猎人枪杀",
                DeathCause::Duel => "决斗落败",
            };
            format!(
                "{} 号（{}）出局：{cause}",
                player.id,
                player.role.display_name()
            )
        }
        GameEvent::GameOver { outcome } => match outcome {
            Outcome::GoodWin => "好人阵营胜利！".to_string(),
            Outcome::WerewolfWin { reason } => match reason {
                WerewolfReason::GodsWiped => "狼人阵营胜利（屠神）！".to_string(),
                WerewolfReason::VillagersWiped => "狼人阵营胜利（屠民）！".to_string(),
            },
        },
    };
    (event.day(), text)
}

impl GameSnapshot {
    /// 从引擎按视角裁剪出快照
    pub fn render(engine: &GameEngine, viewer: Viewer) -> Result<Self, GameError> {
        if let Viewer::Player(id) = viewer {
            GameError::check_seat(id, engine.total_players())?;
        }

        let players = (0..engine.total_players())
            .map(|seat| {
                let player = &engine.players()[seat as usize];
                let role = if is_role_visible(player, viewer, engine) {
                    Some(player.role)
                } else {
                    None
                };
                SnapshotPlayer {
                    id: seat,
                    alive: player.alive,
                    role,
                    can_vote: player.alive && player.can_vote,
                }
            })
            .collect();

        let viewer_is_wolf = match viewer {
            Viewer::God => true,
            Viewer::Player(id) => engine.players()[id as usize].role == Role::Werewolf,
        };
        let logs = engine
            .events()
            .iter()
            .filter(|event| is_visible_event(event, viewer, viewer_is_wolf))
            .map(render_log)
            .map(|(day, text)| SnapshotLog { day, text })
            .collect();

        let outcome = match engine.phase() {
            Phase::Settled { outcome } => Some(outcome.clone()),
            _ => None,
        };

        let checks = match viewer {
            // 验人记录只挂预言家本人
            Viewer::Player(id) if engine.players()[id as usize].role == Role::Seer => {
                engine.players()[id as usize].checks().to_vec()
            }
            _ => Vec::new(),
        };

        Ok(Self {
            day: engine.phase().day(),
            phase: engine.phase().clone(),
            players,
            logs,
            pending: engine.pending().map(|(actor, kind)| PendingWindow {
                actor,
                kind: kind.to_string(),
            }),
            outcome,
            checks,
        })
    }
}

/// 身份可见性：上帝全知；本人可见；死者翻牌与白痴翻牌是公开信息；狼队友互见
fn is_role_visible(player: &PlayerState, viewer: Viewer, engine: &GameEngine) -> bool {
    match viewer {
        Viewer::God => true,
        Viewer::Player(id) => {
            if player.id == id {
                return true;
            }
            if !player.alive || player.revealed_idiot {
                return true;
            }
            let viewer_role = engine.players()[id as usize].role;
            viewer_role == Role::Werewolf && player.role.camp() == Camp::Werewolf
        }
    }
}
