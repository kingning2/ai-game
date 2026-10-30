//! script.rs 的单元测试（事故丢失后重写）。
//!
//! 核心契约：剧本声明（roles/total）必须与引擎板子（`board()`）严格一致——
//! 前端介绍页按剧本渲染，开局按板子发牌，两边对不上就是线上事故。

use std::collections::BTreeMap;

use super::*;
use crate::role::Role;

/// 剧本角色代码 → Role（配比对照用）
fn role_of(code: &str) -> Option<Role> {
    match code {
        "werewolf" => Some(Role::Werewolf),
        "seer" => Some(Role::Seer),
        "witch" => Some(Role::Witch),
        "guard" => Some(Role::Guard),
        "hunter" => Some(Role::Hunter),
        "villager" => Some(Role::Villager),
        "idiot" => Some(Role::Idiot),
        "knight" => Some(Role::Knight),
        _ => None,
    }
}

#[test]
fn every_script_maps_to_consistent_board() {
    for script in SCRIPTS {
        let board = script.board();
        assert_eq!(
            board.total(),
            script.total,
            "剧本 {} 的 total 与板子总人数不一致",
            script.id
        );
        let mut counts: BTreeMap<Role, u8> = BTreeMap::new();
        for role in board.roles() {
            *counts.entry(role).or_insert(0) += 1;
        }
        for (code, count) in script.roles {
            let role =
                role_of(code).unwrap_or_else(|| panic!("剧本 {} 的未知角色代码 {code}", script.id));
            assert_eq!(
                counts.get(&role).copied().unwrap_or(0),
                *count,
                "剧本 {} 的 {} 配比与板子不一致",
                script.id,
                code
            );
        }
        assert!(
            board.validate().is_ok(),
            "剧本 {} 映射的板子未通过引擎校验",
            script.id
        );
    }
}

#[test]
fn script_fields_are_well_formed() {
    let mut ids = Vec::new();
    for script in SCRIPTS {
        assert!(!script.id.is_empty() && !script.name.is_empty());
        assert!(script.total >= 6, "剧本 {} 人数过少", script.id);
        assert!(!script.rules.is_empty() && !script.intro.is_empty());
        let role_total: u8 = script.roles.iter().map(|(_, n)| n).sum();
        assert_eq!(
            role_total, script.total,
            "剧本 {} 的 roles 加总 != total",
            script.id
        );
        ids.push(script.id);
    }
    let unique = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), unique, "剧本 id 重复");
}

#[test]
fn script_lookup_by_id() {
    assert!(script("newbie_six").is_some());
    assert!(script("nine_standard").is_some());
    assert!(script("all_stars").is_some());
    assert!(script("nope").is_none());
}

#[test]
fn unknown_board_id_falls_back_to_nine_standard() {
    // board() 的兜底分支：未知 id 走标准局（引擎侧仍会校验，不会静默开局错误板子）
    assert_eq!(
        script("newbie_six").unwrap().board(),
        crate::board::BoardConfig::newbie_six()
    );
    assert_eq!(
        script("all_stars").unwrap().board(),
        crate::board::BoardConfig::all_stars()
    );
    let fallback = ScriptDef {
        id: "typo_id",
        name: "x",
        total: 9,
        desc: "",
        roles: &[],
        rules: &[],
        intro: &[],
    };
    assert_eq!(fallback.board(), crate::board::BoardConfig::nine_standard());
}
