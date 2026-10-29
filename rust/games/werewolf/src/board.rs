use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::role::Role;

/// 板子配置：各角色的数量表（平民与狼人可多枚，神职 0/1 枚）。
///
/// 预设见 [`BoardConfig::nine_standard`] 等；自定义板子用 [`BoardConfig::new`]
/// 构建后由引擎在开局时校验（见 `engine.rs`）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "lowercase")]
pub struct BoardConfig {
    pub werewolves: u8,
    pub villagers: u8,
    pub seer: u8,
    pub witch: u8,
    pub guard: u8,
    pub hunter: u8,
    pub idiot: u8,
    pub knight: u8,
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self::nine_standard()
    }
}

impl BoardConfig {
    /// 新手 6 人局：2 狼 + 预言家 + 女巫 + 2 民（对齐前端 `rookie6` 板）
    pub fn newbie_six() -> Self {
        Self {
            werewolves: 2,
            villagers: 2,
            seer: 1,
            witch: 1,
            guard: 0,
            hunter: 0,
            idiot: 0,
            knight: 0,
        }
    }

    /// 经典 9 人标准局：3 狼 + 预女猎 + 3 民（屠边规则的标准板子）
    pub fn nine_standard() -> Self {
        Self {
            werewolves: 3,
            villagers: 3,
            seer: 1,
            witch: 1,
            guard: 0,
            hunter: 1,
            idiot: 0,
            knight: 0,
        }
    }

    /// 全明星 12 人局：4 狼 + 预言家/女巫/守卫/猎人 + 4 民（与前端「全明星 12 人局」对齐）
    pub fn all_stars() -> Self {
        Self {
            werewolves: 4,
            villagers: 4,
            seer: 1,
            witch: 1,
            guard: 1,
            hunter: 1,
            idiot: 0,
            knight: 0,
        }
    }

    /// 角色总数（= 玩家座位数）
    pub fn total(&self) -> u8 {
        self.werewolves
            + self.villagers
            + self.seer
            + self.witch
            + self.guard
            + self.hunter
            + self.idiot
            + self.knight
    }

    /// 展开成角色池（发牌用的多重集合）
    pub fn roles(&self) -> Vec<Role> {
        let mut roles = Vec::with_capacity(self.total() as usize);
        let push = |roles: &mut Vec<Role>, count: u8, role: Role| {
            for _ in 0..count {
                roles.push(role);
            }
        };
        push(&mut roles, self.werewolves, Role::Werewolf);
        push(&mut roles, self.seer, Role::Seer);
        push(&mut roles, self.witch, Role::Witch);
        push(&mut roles, self.guard, Role::Guard);
        push(&mut roles, self.hunter, Role::Hunter);
        push(&mut roles, self.idiot, Role::Idiot);
        push(&mut roles, self.knight, Role::Knight);
        push(&mut roles, self.villagers, Role::Villager);
        roles
    }

    /// 校验：至少 1 狼（没有狼好人自动赢，对局无意义）、至少 1 个好人、
    /// 神职每项至多 1 枚（本引擎按单枚神职实现夜晚窗口）
    pub fn validate(&self) -> Result<(), BoardError> {
        if self.werewolves == 0 {
            return Err(BoardError::NoWerewolf);
        }
        if self.total() <= self.werewolves {
            return Err(BoardError::NoGood);
        }
        for (name, count) in [
            ("seer", self.seer),
            ("witch", self.witch),
            ("guard", self.guard),
            ("hunter", self.hunter),
            ("idiot", self.idiot),
            ("knight", self.knight),
        ] {
            if count > 1 {
                return Err(BoardError::DuplicateGod(name.to_string()));
            }
        }
        Ok(())
    }

    /// 各角色数量视图（日志/前端展示用）
    pub fn counts(&self) -> BTreeMap<&'static str, u8> {
        BTreeMap::from([
            ("werewolf", self.werewolves),
            ("seer", self.seer),
            ("witch", self.witch),
            ("guard", self.guard),
            ("hunter", self.hunter),
            ("idiot", self.idiot),
            ("knight", self.knight),
            ("villager", self.villagers),
        ])
    }
}

use crate::error::BoardError;

#[cfg(test)]
mod tests {
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
}
