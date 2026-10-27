use crate::action::Action;
use crate::board::{BoardConfig, BoardError};
use crate::error::GameError;
use crate::event::{GameEvent, NightDeath};
use crate::phase::{
    CheckRecord, CheckResult, DeathCause, NightStage, Outcome, Phase, PlayerId, PlayerRole,
    WerewolfReason,
};
use crate::player::PlayerState;
use crate::role::{Camp, Role};

/// 随机数发生器：xorshift64（发牌洗牌用；seed 相同 ⇒ 发牌相同，可复现测试）
struct Rng(u64);

impl Rng {
    /// splitmix64 雪崩混淆：相邻 seed（如 7/8）也能产生完全不同的序列
    fn from_seed(seed: u64) -> Self {
        let mut z = seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Self((z ^ (z >> 31)) | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Fisher–Yates 洗牌
    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = (self.next_u64() % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }
}

/// 女巫药水持有量（各一瓶，整局有效）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Potions {
    save: bool,
    poison: bool,
}

/// 单晚夜间动作累计（天亮结算后清空）
#[derive(Debug, Clone, Copy, Default)]
struct NightRecord {
    /// 守卫守护目标
    protected: Option<PlayerId>,
    /// 狼队刀口
    killed: Option<PlayerId>,
    /// 女巫是否用了解药
    saved: bool,
    /// 女巫毒杀目标
    poisoned: Option<PlayerId>,
}

/// 狼队夜聊轮转游标：从第一个活狼起按座位序绕一圈，回到起点即结束
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WolfChatState {
    cursor: PlayerId,
    start: PlayerId,
}

/// 狼人杀游戏引擎：纯状态机，零 IO。
///
/// 推进方式：调用方读 [`GameEngine::pending`] 了解当前等谁做什么，
/// 喂入合法 [`Action`]；引擎校验后推进状态机并返回增量事件流。
/// 对外可见的保密视图见 [`GameEngine::snapshot`]（身份只在上帝视角/翻牌后可见）。
#[derive(Debug, Clone)]
pub struct GameEngine {
    /// 座位数（board.total()）
    total: u8,
    players: Vec<PlayerState>,
    phase: Phase,
    /// 完整上帝日志
    events: Vec<GameEvent>,
    potions: Potions,
    /// 昨晚守护目标（守卫不能连守同一人）
    last_protected: Option<PlayerId>,
    night: NightRecord,
    /// 本轮白天投票的票箱（逐个收集，结算后清空）
    votes: Vec<Option<PlayerId>>,
    /// 骑士决斗窗口本轮白天是否已出现（一次）
    duel_offered: bool,
    /// 狼队夜聊轮转（仅 [`NightStage::WerewolfChat`] 段非空）
    wolf_chat: Option<WolfChatState>,
}

impl GameEngine {
    /// 开局：校验板子、洗牌发牌。`seed` 相同 ⇒ 发牌相同（可复现）
    pub fn new(board: BoardConfig, seed: u64) -> Result<Self, BoardError> {
        board.validate()?;
        let mut roles = board.roles();
        Rng::from_seed(seed).shuffle(&mut roles);
        let players = roles
            .into_iter()
            .enumerate()
            .map(|(i, role)| PlayerState::new(i as PlayerId, role))
            .collect();
        let mut engine = Self {
            total: board.total(),
            players,
            phase: Phase::Night {
                day: 1,
                stage: NightStage::Guard,
            },
            events: Vec::new(),
            potions: Potions {
                save: board.witch > 0,
                poison: board.witch > 0,
            },
            last_protected: None,
            night: NightRecord::default(),
            votes: Vec::new(),
            duel_offered: false,
            wolf_chat: None,
        };
        engine.push(GameEvent::GameStarted {
            players: engine.total,
            board: board
                .counts()
                .into_iter()
                .filter(|(_, count)| *count > 0)
                .map(|(name, count)| (name.to_string(), count))
                .collect(),
        });
        engine.enter_night(1);
        Ok(engine)
    }

    /// 当前阶段
    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// 完整事件流（上帝日志）
    pub fn events(&self) -> &[GameEvent] {
        &self.events
    }

    /// 对局是否已结束
    pub fn settled(&self) -> bool {
        matches!(self.phase, Phase::Settled { .. })
    }

    /// 座位数
    pub fn total_players(&self) -> u8 {
        self.total
    }

    /// 玩家内部状态（供快照裁剪；身份保密由快照层负责，不要直接外发）
    pub fn players(&self) -> &[PlayerState] {
        &self.players
    }

    fn push(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    fn player(&self, id: PlayerId) -> &PlayerState {
        &self.players[id as usize]
    }

    fn player_mut(&mut self, id: PlayerId) -> &mut PlayerState {
        &mut self.players[id as usize]
    }

    /// 某角色的唯一存活玩家（夜晚窗口的 actor）；已死则窗口应被跳过
    fn find_role(&self, role: Role) -> Option<PlayerId> {
        self.players
            .iter()
            .find(|p| p.role == role && p.alive)
            .map(|p| p.id)
    }

    /// 存活计数：(狼数, 神职数, 平民数)
    fn counts(&self) -> (u8, u8, u8) {
        self.players
            .iter()
            .filter(|p| p.alive)
            .fold((0, 0, 0), |(w, g, v), p| match p.role.camp() {
                Camp::Werewolf => (w + 1, g, v),
                Camp::Good if p.role.is_god() => (w, g + 1, v),
                Camp::Good => (w, g, v + 1),
            })
    }

    /// 胜负判定（屠边规则）。分出胜负则广播事件并置为 Settled
    fn check_outcome(&mut self) -> bool {
        let (wolves, gods, villagers) = self.counts();
        let outcome = if wolves == 0 {
            Some(Outcome::GoodWin)
        } else if wolves >= gods {
            Some(Outcome::WerewolfWin {
                reason: WerewolfReason::GodsWiped,
            })
        } else if wolves >= villagers {
            Some(Outcome::WerewolfWin {
                reason: WerewolfReason::VillagersWiped,
            })
        } else {
            None
        };
        match outcome {
            Some(outcome) => {
                self.push(GameEvent::GameOver {
                    outcome: outcome.clone(),
                });
                self.phase = Phase::Settled { outcome };
                true
            }
            None => false,
        }
    }

    /// 当前等待谁做什么：`(actor 座位, 窗口描述)`；对局结束返回 `None`。
    /// 夜晚窗口在进入时已自动跳过死角色（见 [`Self::skip_dead_stages`]），
    /// 因此这里返回的等待者保证存活
    pub fn pending(&self) -> Option<(PlayerId, &'static str)> {
        match &self.phase {
            Phase::Night { stage, .. } => match stage {
                NightStage::WerewolfChat => {
                    let seat = self.wolf_chat.as_ref().map(|chat| chat.cursor)?;
                    Some((seat, "狼人夜聊"))
                }
                _ => {
                    let seat = self.find_role(stage.role())?;
                    Some((seat, "夜晚行动"))
                }
            },
            Phase::DaySpeech { next_speaker, .. } => self
                .next_alive_speaker(*next_speaker)
                .map(|id| (id, "白天发言")),
            Phase::KnightDuel { knight, .. } => Some((*knight, "骑士决斗")),
            Phase::DayVote { .. } => self
                .players
                .iter()
                .find(|p| p.alive && p.can_vote)
                .map(|p| (p.id, "投票")),
            Phase::HunterShot { hunter, .. } => Some((*hunter, "猎人开枪")),
            Phase::Settled { .. } => None,
        }
    }

    /// 从 `from` 座位起（含）找下一个活人座位
    fn next_alive_speaker(&self, from: PlayerId) -> Option<PlayerId> {
        (0..self.total)
            .map(|offset| (from + offset) % self.total)
            .find(|seat| self.player(*seat).alive)
    }

    /// 喂入一个动作，推进状态机，返回本次产生的增量事件。
    /// 校验失败时不留下任何半截事件
    pub fn apply(&mut self, actor: PlayerId, action: Action) -> Result<Vec<GameEvent>, GameError> {
        if self.settled() {
            return Err(GameError::GameOver);
        }
        let before = self.events.len();
        let result = match self.phase.clone() {
            Phase::Night { day, stage } => self.apply_night(actor, action, day, stage),
            Phase::DaySpeech { day, next_speaker } => {
                self.apply_speech(actor, action, day, next_speaker)
            }
            Phase::KnightDuel { day, knight } => self.apply_duel(actor, action, day, knight),
            Phase::DayVote { day } => self.apply_vote(actor, action, day),
            Phase::HunterShot { day, hunter } => self.apply_hunter_shot(actor, action, day, hunter),
            Phase::Settled { .. } => Err(GameError::GameOver),
        };
        match result {
            Ok(()) => Ok(self.events[before..].to_vec()),
            Err(err) => {
                self.events.truncate(before);
                Err(err)
            }
        }
    }

    // ---------- 校验工具 ----------

    /// 夜晚窗口的角色校验：actor 必须是该角色的唯一存活者
    fn check_night_actor(&self, actor: PlayerId, role: Role) -> Result<(), GameError> {
        match self.find_role(role) {
            Some(expected) if expected == actor => Ok(()),
            Some(expected) => Err(GameError::NotYourTurn {
                expected,
                actual: actor,
            }),
            None => Err(GameError::WrongPhase("该角色窗口因出局已跳过")),
        }
    }

    fn check_target_alive(&self, target: PlayerId) -> Result<(), GameError> {
        GameError::check_seat(target, self.total)?;
        if !self.player(target).alive {
            return Err(GameError::TargetDead(target));
        }
        Ok(())
    }

    /// 狼队共享窗口：任意存活狼都能提交刀口（区别于 [`Self::check_night_actor`] 的唯一者校验）
    fn check_wolf_actor(&self, actor: PlayerId) -> Result<(), GameError> {
        let p = self.player(actor);
        if !p.alive || p.role != Role::Werewolf {
            let expected = self.find_role(Role::Werewolf).unwrap_or(actor);
            return Err(GameError::NotYourTurn {
                expected,
                actual: actor,
            });
        }
        Ok(())
    }

    // ---------- 夜晚 ----------

    /// 开启狼队夜聊：游标定位到第一个活狼（存活狼至少 1，否则该段已被跳过）
    fn open_wolf_chat(&mut self) {
        if let Some(start) = self.find_role(Role::Werewolf) {
            self.wolf_chat = Some(WolfChatState {
                cursor: start,
                start,
            });
        }
    }

    /// 狼队夜聊：当前游标狼发言/沉默，然后轮到下一位活狼；
    /// 绕回起点则夜聊结束，进入刀人窗口
    fn apply_wolf_chat(
        &mut self,
        actor: PlayerId,
        text: Option<String>,
        day: u32,
    ) -> Result<(), GameError> {
        let chat = match self.wolf_chat {
            Some(chat) if chat.cursor == actor => chat,
            Some(chat) => {
                return Err(GameError::NotYourTurn {
                    expected: chat.cursor,
                    actual: actor,
                });
            }
            None => return Err(GameError::WrongPhase("狼聊窗口未开启")),
        };
        let speaker = self.player(actor).named();
        if let Some(text) = text {
            let text = text.trim();
            if text.is_empty() {
                return Err(GameError::EmptySpeech);
            }
            self.push(GameEvent::WolfChatSpoke {
                day,
                speaker,
                text: text.to_string(),
            });
        } else {
            self.push(GameEvent::WolfChatPassed { day, speaker });
        }
        // 下一位活狼（只在狼之间轮转）；绕回起点（含单狼自轮）则夜聊结束，进刀人窗口
        let next = (0..self.total)
            .map(|offset| (chat.cursor + 1 + offset) % self.total)
            .find(|seat| {
                let p = self.player(*seat);
                p.alive && p.role == Role::Werewolf
            })
            .unwrap_or(chat.start);
        if next == chat.start {
            self.wolf_chat = None;
            self.skip_dead_stages(NightStage::Werewolves, day);
        } else {
            self.wolf_chat = Some(WolfChatState {
                cursor: next,
                start: chat.start,
            });
        }
        Ok(())
    }

    fn apply_night(
        &mut self,
        actor: PlayerId,
        action: Action,
        day: u32,
        stage: NightStage,
    ) -> Result<(), GameError> {
        let logged = action.clone();
        match (stage, action) {
            (NightStage::Guard, Action::GuardProtect { target }) => {
                self.check_night_actor(actor, Role::Guard)?;
                self.apply_guard(target)?;
            }
            (NightStage::WerewolfChat, action) => {
                let Action::WolfChat { text } = action else {
                    return Err(GameError::WrongPhase("狼聊窗口只接受 WolfChat"));
                };
                // 狼聊自带专用事件与推进（游标绕回起点 → skip_dead_stages 进刀人窗口），
                // 不走尾部的统一 NightAction 上报与 advance_night
                self.apply_wolf_chat(actor, text, day)?;
                return Ok(());
            }
            (NightStage::Werewolves, Action::WerewolfKill { target }) => {
                self.check_wolf_actor(actor)?;
                self.check_target_alive(target)?;
                self.night.killed = Some(target);
            }
            (NightStage::Witch, Action::WitchAct { save, poison }) => {
                self.check_night_actor(actor, Role::Witch)?;
                self.apply_witch(actor, save, poison)?;
            }
            (NightStage::Seer, Action::SeerCheck { target }) => {
                self.check_night_actor(actor, Role::Seer)?;
                self.apply_seer(actor, target, day)?;
            }
            _ => return Err(GameError::WrongPhase("夜晚动作与当前窗口不符")),
        }
        self.push(GameEvent::NightAction {
            actor: self.player(actor).named(),
            action: logged,
        });
        self.advance_night(day);
        Ok(())
    }

    fn apply_guard(&mut self, target: Option<PlayerId>) -> Result<(), GameError> {
        if let Some(target) = target {
            self.check_target_alive(target)?;
            if self.last_protected == Some(target) {
                return Err(GameError::GuardRepeat);
            }
            self.night.protected = Some(target);
        }
        Ok(())
    }

    fn apply_witch(
        &mut self,
        witch: PlayerId,
        save: bool,
        poison: Option<PlayerId>,
    ) -> Result<(), GameError> {
        if save && poison.is_some() {
            return Err(GameError::WitchBothPotions);
        }
        if save {
            if !self.potions.save {
                return Err(GameError::PotionDepleted("解药"));
            }
            let kill = self.night.killed.ok_or(GameError::NoKillToSave)?;
            self.check_target_alive(kill)?;
            self.night.saved = true;
            self.potions.save = false;
        }
        if let Some(target) = poison {
            self.check_target_alive(target)?;
            if !self.potions.poison {
                return Err(GameError::PotionDepleted("毒药"));
            }
            if target == witch {
                return Err(GameError::SelfTarget("女巫"));
            }
            self.night.poisoned = Some(target);
            self.potions.poison = false;
        }
        Ok(())
    }

    fn apply_seer(&mut self, seer: PlayerId, target: PlayerId, day: u32) -> Result<(), GameError> {
        self.check_target_alive(target)?;
        if target == seer {
            return Err(GameError::SelfTarget("预言家"));
        }
        let result = match self.player(target).role.camp() {
            Camp::Werewolf => CheckResult::Werewolf,
            Camp::Good => CheckResult::Good,
        };
        self.player_mut(seer).checks.push(CheckRecord {
            day,
            target,
            result,
        });
        Ok(())
    }

    /// 夜晚推进：当前窗口动作已收 → 下一段（跳过无存活角色的段）；最后一段 → 结算
    fn advance_night(&mut self, day: u32) {
        if let Phase::Night { stage, .. } = self.phase {
            match stage.next() {
                Some(next) => self.skip_dead_stages(next, day),
                None => self.resolve_night(day),
            }
        }
    }

    /// 夜晚结算：刀口 + 守护 + 解药 + 毒药 → 死亡名单 → 猎人窗口/入昼
    ///
    /// 规则要点：
    /// - 被刀未守未救 ⇒ 死；被刀且被守 ⇒ 平安；被刀且被救 ⇒ 平安；
    /// - **奶穿**：被刀且既被守又被救 ⇒ 死（双保险冲突）
    /// - 毒杀独立结算，被毒猎人不能开枪
    fn resolve_night(&mut self, day: u32) {
        let NightRecord {
            protected,
            killed,
            saved,
            poisoned,
        } = self.night;
        let mut deaths: Vec<NightDeath> = Vec::new();

        if let Some(kill) = killed {
            let guarded = protected == Some(kill);
            // (false,false) 刀死；(true,true) 奶穿死；其余平安
            if guarded == saved {
                deaths.push(NightDeath {
                    player: kill,
                    cause: DeathCause::Slain,
                });
            }
        }
        if let Some(poison) = poisoned
            && !deaths.iter().any(|d| d.player == poison)
        {
            deaths.push(NightDeath {
                player: poison,
                cause: DeathCause::Poisoned,
            });
        }

        self.last_protected = protected;
        self.night = NightRecord::default();

        // 被刀死的猎人可开枪（先收集，后击杀）
        let hunter_shots: Vec<PlayerId> = deaths
            .iter()
            .filter(|d| d.cause == DeathCause::Slain)
            .map(|d| d.player)
            .filter(|id| self.player(*id).role == Role::Hunter)
            .collect();

        for death in &deaths {
            self.players[death.player as usize].alive = false;
        }
        for death in &deaths {
            self.push(GameEvent::PlayerDied {
                day,
                player: self.players[death.player as usize].named(),
                cause: death.cause,
            });
        }

        let game_over = self.check_outcome();
        self.push(GameEvent::Dawn { day, deaths });
        if game_over {
            return;
        }
        match hunter_shots.first() {
            Some(hunter) => {
                self.phase = Phase::HunterShot {
                    day,
                    hunter: *hunter,
                }
            }
            None => self.begin_day(day),
        }
    }

    /// 入昼：从首个死者下一位开始发言（平安夜从 0 号开始）
    fn begin_day(&mut self, day: u32) {
        let first_death = self.events.iter().rev().find_map(|event| match event {
            GameEvent::Dawn { day: d, deaths } if *d == day && !deaths.is_empty() => {
                Some(deaths[0].player)
            }
            _ => None,
        });
        let next_speaker = match first_death {
            Some(dead) => (dead + 1) % self.total,
            None => 0,
        };
        self.phase = Phase::DaySpeech { day, next_speaker };
        self.duel_offered = false;
    }

    // ---------- 白天 ----------

    fn apply_speech(
        &mut self,
        actor: PlayerId,
        action: Action,
        day: u32,
        next_speaker: PlayerId,
    ) -> Result<(), GameError> {
        let expected = self
            .next_alive_speaker(next_speaker)
            .ok_or(GameError::WrongPhase("无存活发言者"))?;
        if actor != expected {
            return Err(GameError::NotSpeakerTurn {
                expected,
                actual: actor,
            });
        }
        let Action::Speech { text } = action else {
            return Err(GameError::WrongPhase("发言窗口只接受 Speech"));
        };
        if text.trim().is_empty() {
            return Err(GameError::EmptySpeech);
        }
        self.push(GameEvent::SpeechMade {
            day,
            speaker: actor,
            text,
        });
        self.advance_speech(day, actor);
        Ok(())
    }

    /// 发言轮转：今天已发言数 < 存活数 ⇒ 下一位；否则进入决斗/投票
    fn advance_speech(&mut self, day: u32, just_spoke: PlayerId) {
        let alive: Vec<PlayerId> = (0..self.total)
            .filter(|seat| self.player(*seat).alive)
            .collect();
        let spoken = self.spoken_today(day);
        if spoken.len() < alive.len() {
            // 不用 expect：找不到下一位（理论上不可能，防御式处理）直接推进到投票阶段
            if let Some(next) = self.next_alive_speaker((just_spoke + 1) % self.total) {
                self.phase = Phase::DaySpeech {
                    day,
                    next_speaker: next,
                };
                return;
            }
        }
        self.after_speech(day);
    }

    /// 今天已发言的座位列表（从事件流统计）
    fn spoken_today(&self, day: u32) -> Vec<PlayerId> {
        self.events
            .iter()
            .filter_map(|event| match event {
                GameEvent::SpeechMade {
                    day: d, speaker, ..
                } if *d == day => Some(*speaker),
                _ => None,
            })
            .collect()
    }

    /// 发言结束 → 骑士决斗窗口（存活且未用过）→ 投票
    fn after_speech(&mut self, day: u32) {
        if !self.duel_offered
            && let Some(knight) = self.find_role(Role::Knight)
            && !self.player(knight).duel_used
        {
            self.duel_offered = true;
            self.phase = Phase::KnightDuel { day, knight };
            return;
        }
        self.phase = Phase::DayVote { day };
    }

    fn apply_duel(
        &mut self,
        actor: PlayerId,
        action: Action,
        day: u32,
        knight: PlayerId,
    ) -> Result<(), GameError> {
        if actor != knight {
            return Err(GameError::NotYourTurn {
                expected: knight,
                actual: actor,
            });
        }
        let Action::KnightDuel { target } = action else {
            return Err(GameError::WrongPhase("决斗窗口只接受 KnightDuel"));
        };
        self.player_mut(knight).duel_used = true;
        if let Some(target) = target {
            self.check_target_alive(target)?;
            let target_is_wolf = self.player(target).role.camp() == Camp::Werewolf;
            let loser = if target_is_wolf {
                self.player(target).named()
            } else {
                // 骑士赌输：骑士死（决斗死亡不触发猎人开枪）
                self.player(knight).named()
            };
            self.kill(loser.id, day, DeathCause::Duel);
            self.push(GameEvent::KnightDuelResolved {
                day,
                knight,
                target,
                target_is_wolf,
                loser,
            });
            if self.check_outcome() {
                return Ok(());
            }
        }
        self.phase = Phase::DayVote { day };
        Ok(())
    }

    /// 白天统一击杀：置死 + 翻牌广播
    fn kill(&mut self, target: PlayerId, day: u32, cause: DeathCause) {
        let player = self.player_mut(target);
        player.alive = false;
        let named = player.named();
        self.push(GameEvent::PlayerDied {
            day,
            player: named,
            cause,
        });
    }

    fn apply_vote(&mut self, actor: PlayerId, action: Action, day: u32) -> Result<(), GameError> {
        let Action::Vote { target } = action else {
            return Err(GameError::WrongPhase("投票窗口只接受 Vote"));
        };
        // 投票不强制顺序：任何未投票的活人都可以投；已投/已死/无权 → 拒绝
        let voter = self.player(actor);
        if !voter.alive {
            return Err(GameError::TargetDead(actor));
        }
        if !voter.can_vote {
            return Err(GameError::AlreadyVoted(actor));
        }
        if let Some(target) = target {
            self.check_target_alive(target)?;
        }
        self.votes.push(target);
        self.player_mut(actor).can_vote = false;

        // 还有未投票的活人吗？
        if self.players.iter().any(|p| p.alive && p.can_vote) {
            return Ok(());
        }
        self.resolve_vote(day);
        Ok(())
    }

    /// 结算投票：统计 → 平票无人放逐 / 放逐（白痴翻牌 / 猎人窗口）→ 胜负 → 入夜
    fn resolve_vote(&mut self, day: u32) {
        let mut counts: Vec<(PlayerId, u8)> = Vec::new();
        for vote in self.votes.iter().flatten() {
            match counts.iter_mut().find(|(seat, _)| seat == vote) {
                Some(entry) => entry.1 += 1,
                None => counts.push((*vote, 1)),
            }
        }
        self.votes.clear();

        let max = counts.iter().map(|(_, count)| *count).max().unwrap_or(0);
        let top: Vec<PlayerId> = counts
            .iter()
            .filter(|(_, count)| *count == max && max > 0)
            .map(|(seat, _)| *seat)
            .collect();

        // 恢复所有活人投票权（白痴除外，下面对其单独处理）
        for player in &mut self.players {
            if player.alive {
                player.can_vote = true;
            }
        }

        if top.len() != 1 {
            self.push(GameEvent::VoteResolved {
                day,
                counts,
                tie: true,
                exiled: None,
            });
            self.check_outcome();
            self.enter_night(day + 1);
            return;
        }

        let exiled = top[0];
        self.push(GameEvent::VoteResolved {
            day,
            counts,
            tie: false,
            exiled: Some(self.player(exiled).named()),
        });

        // 白痴翻牌：放逐免死、失去投票权（revealed_idiot 标记）
        if self.player(exiled).role == Role::Idiot {
            let player = self.player_mut(exiled);
            player.can_vote = false;
            player.revealed_idiot = true;
            self.push(GameEvent::IdiotRevealed { day, idiot: exiled });
            self.check_outcome();
            self.enter_night(day + 1);
            return;
        }

        self.kill(exiled, day, DeathCause::Exiled);
        if self.check_outcome() {
            return;
        }
        // 被放逐猎人可开枪
        if self.players[exiled as usize].role == Role::Hunter {
            self.phase = Phase::HunterShot {
                day,
                hunter: exiled,
            };
            return;
        }
        self.enter_night(day + 1);
    }

    /// 进入新夜晚：广播天黑，并把起始窗口定位到第一个存活角色
    /// （板子没有守卫 / 该角色昨夜阵亡都会被跳过）
    fn enter_night(&mut self, day: u32) {
        self.push(GameEvent::NightBegan { day });
        self.skip_dead_stages(NightStage::Guard, day);
    }

    /// 从 `stage` 起找到第一个「存活角色」的窗口；后面全无存活角色则直接结算
    /// （理论不可达：好人全灭时胜负早已判定）
    fn skip_dead_stages(&mut self, stage: NightStage, day: u32) {
        let mut stage = stage;
        loop {
            if self.find_role(stage.role()).is_some() {
                self.phase = Phase::Night { day, stage };
                if stage == NightStage::WerewolfChat {
                    self.open_wolf_chat();
                }
                return;
            }
            match stage.next() {
                Some(next) => stage = next,
                None => {
                    self.resolve_night(day);
                    return;
                }
            }
        }
    }

    fn apply_hunter_shot(
        &mut self,
        actor: PlayerId,
        action: Action,
        day: u32,
        hunter: PlayerId,
    ) -> Result<(), GameError> {
        if actor != hunter {
            return Err(GameError::NotYourTurn {
                expected: hunter,
                actual: actor,
            });
        }
        let Action::HunterShot { target } = action else {
            return Err(GameError::WrongPhase("猎人窗口只接受 HunterShot"));
        };
        let hunter_named = self.player(hunter).named();
        let mut shot: Option<PlayerRole> = None;
        if let Some(target) = target {
            self.check_target_alive(target)?;
            self.kill(target, day, DeathCause::Shot);
            shot = Some(self.players[target as usize].named());
        }
        self.push(GameEvent::HunterShotFired {
            day,
            hunter: hunter_named,
            target: shot,
        });
        if self.check_outcome() {
            return Ok(());
        }
        // 级联：枪死者也是猎人 ⇒ 新窗口
        if let Some(target) = target
            && self.players[target as usize].role == Role::Hunter
        {
            self.phase = Phase::HunterShot {
                day,
                hunter: target,
            };
            return Ok(());
        }
        // 窗口去向：放逐触发 ⇒ 下一夜；夜晚触发 ⇒ 回到白天发言
        let vote_done = self
            .events
            .iter()
            .any(|event| matches!(event, GameEvent::VoteResolved { day: d, .. } if *d == day));
        if vote_done {
            self.enter_night(day + 1);
        } else {
            self.begin_day(day);
        }
        Ok(())
    }
}
