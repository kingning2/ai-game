//! `autoplay.rs` 的单元测试：工具契约、tool_calls 解析、文本兜底解析、规则兜底。

use super::*;
use werewolf::PlayerRole;

/// 标准决策上下文：3 号位狼人，存活 1-5
fn context(kind: &'static str, role: Role) -> DecisionContext {
    DecisionContext {
        seat: 3,
        name: "测试选手".into(),
        role,
        wolf_mates: if role == Role::Werewolf {
            vec![5]
        } else {
            Vec::new()
        },
        alive: vec![1, 2, 3, 4, 5],
        history: String::new(),
        checks: Vec::new(),
        memory: String::new(),
        kind,
        day: 1,
    }
}

fn call(name: &str, arguments: &str) -> ToolCall {
    ToolCall {
        id: "call_1".into(),
        name: name.into(),
        arguments: arguments.into(),
    }
}

// ===== tool_for：场景 → 工具契约 =====

#[test]
fn tool_names_match_decision_kinds() {
    assert_eq!(tool_for(&context("白天发言", Role::Villager)).name, "speak");
    assert_eq!(tool_for(&context("投票", Role::Villager)).name, "vote");
    assert_eq!(
        tool_for(&context("狼人夜聊", Role::Werewolf)).name,
        "wolf_chat"
    );
    assert_eq!(
        tool_for(&context("狼人行动", Role::Werewolf)).name,
        "wolf_kill"
    );
    assert_eq!(
        tool_for(&context("夜晚行动", Role::Seer)).name,
        "seer_check"
    );
    assert_eq!(
        tool_for(&context("夜晚行动", Role::Witch)).name,
        "witch_act"
    );
    assert_eq!(
        tool_for(&context("夜晚行动", Role::Guard)).name,
        "guard_protect"
    );
    assert_eq!(
        tool_for(&context("骑士决斗", Role::Knight)).name,
        "knight_duel"
    );
    assert_eq!(
        tool_for(&context("猎人开枪", Role::Hunter)).name,
        "hunter_shot"
    );
}

/// schema 是合法 JSON 且带属性与描述（网关/模型都依赖；宏生成保证结构）
#[test]
fn tool_schemas_are_valid_json() {
    for role in [Role::Seer, Role::Witch, Role::Guard, Role::Werewolf] {
        let tool = tool_for(&context("夜晚行动", role));
        assert_eq!(tool.parameters["type"], "object");
        assert!(
            !tool.parameters["properties"]
                .as_object()
                .unwrap()
                .is_empty()
        );
    }
    // 必填目标场景带 required；纯可选场景（守护）没有 required 键
    assert_eq!(
        tool_for(&context("夜晚行动", Role::Seer)).parameters["required"],
        serde_json::json!(["target"])
    );
    assert!(
        tool_for(&context("夜晚行动", Role::Guard))
            .parameters
            .get("required")
            .is_none()
    );
}

// ===== parse_tool_action =====

#[test]
fn tool_call_parses_to_action() {
    let ctx = context("白天发言", Role::Villager);
    let action = parse_tool_action(
        &ctx,
        &[call(
            "speak",
            r#"{"text":"3号昨天跳预言家但验人逻辑有漏洞，我认为他是悍跳狼，今天建议先票他观察水像。"}"#,
        )],
    )
    .unwrap();
    assert!(matches!(action, Action::Speech { .. }));

    let ctx = context("投票", Role::Villager);
    let action = parse_tool_action(&ctx, &[call("vote", r#"{"target":4}"#)]).unwrap();
    assert_eq!(action, Action::Vote { target: Some(4) });

    // 弃票 null
    let action = parse_tool_action(&ctx, &[call("vote", r#"{"target":null}"#)]).unwrap();
    assert_eq!(action, Action::Vote { target: None });
}

/// 工具名不匹配当前场景 → None（交文本兜底 / 纠错重问）
#[test]
fn wrong_tool_name_is_none() {
    let ctx = context("白天发言", Role::Villager);
    assert!(parse_tool_action(&ctx, &[call("vote", r#"{"target":2}"#)]).is_none());
}

/// 越界 / 指向自己 / 非法参数 → 对应动作解析失败
#[test]
fn invalid_targets_rejected() {
    let ctx = context("投票", Role::Villager);
    // 目标是自己（3 号）→ target_field 为 None → 弃票兜底（Vote 可空）
    let action = parse_tool_action(&ctx, &[call("vote", r#"{"target":3}"#)]).unwrap();
    assert_eq!(action, Action::Vote { target: None });
    // 必填目标的狼刀指向死者/越界 → None；刀队友合法通过（引擎层拒绝，走纠错回喂）
    let ctx = context("狼人行动", Role::Werewolf);
    assert!(parse_tool_action(&ctx, &[call("wolf_kill", r#"{"target":99}"#)]).is_none());
    let action = parse_tool_action(&ctx, &[call("wolf_kill", r#"{"target":5}"#)]).unwrap();
    assert_eq!(action, Action::WerewolfKill { target: 5 });
}

/// 女巫药水：save + poison 双字段
#[test]
fn witch_act_from_tool_call() {
    let ctx = context("夜晚行动", Role::Witch);
    let action =
        parse_tool_action(&ctx, &[call("witch_act", r#"{"save":true,"target":null}"#)]).unwrap();
    assert_eq!(
        action,
        Action::WitchAct {
            save: true,
            poison: None
        }
    );
}

// ===== 文本 JSON 兜底解析 =====

#[test]
fn text_json_still_parses() {
    let ctx = context("白天发言", Role::Villager);
    let action = parse_action(
        &ctx,
        r#"前言 {"text":"我认为2号发言有问题，理由有三。"} 后语"#,
    )
    .unwrap();
    assert!(matches!(action, Action::Speech { .. }));
}

#[test]
fn garbage_text_is_none() {
    let ctx = context("白天发言", Role::Villager);
    assert!(parse_action(&ctx, "我不会用 JSON").is_none());
}

// ===== 规则兜底 =====

/// 兜底发言绝不能暴露身份（旧文案「我是狼人，过」是自爆事故）
#[test]
fn fallback_speech_never_leaks_role() {
    for role in [Role::Werewolf, Role::Seer, Role::Witch, Role::Villager] {
        let ctx = context("白天发言", role);
        let Action::Speech { text } = fallback_action(&ctx).unwrap() else {
            panic!("白天发言兜底必须是 Speech");
        };
        for role_name in [
            "狼人",
            "预言家",
            "女巫",
            "守卫",
            "猎人",
            "白痴",
            "骑士",
            "平民",
        ] {
            assert!(!text.contains(role_name), "兜底发言泄身份：{text}");
        }
    }
}

/// 狼刀兜底不刀队友
#[test]
fn fallback_wolf_kill_avoids_mates() {
    let ctx = context("狼人行动", Role::Werewolf);
    let Action::WerewolfKill { target } = fallback_action(&ctx).unwrap() else {
        panic!("狼刀兜底必须是 WerewolfKill");
    };
    assert!(target != 5 && target != 3);
}

// ===== push_correction：纠错消息与 tool_calls 配对 =====

#[test]
fn correction_pairs_with_tool_calls() {
    let calls = vec![call("speak", "{}")];
    let mut history = vec![ChatMessage::user("开局")];
    push_correction(&mut history, &calls, "引擎拒绝");
    let last = history.last().unwrap();
    assert_eq!(last.role, agent::message::Role::Tool);
    assert_eq!(last.tool_results[0].call_id, "call_1");
    assert_eq!(last.tool_results[0].content, "引擎拒绝");
}

#[test]
fn correction_without_calls_is_user_message() {
    let mut history = vec![ChatMessage::user("开局")];
    push_correction(&mut history, &[], "解析失败");
    let last = history.last().unwrap();
    assert_eq!(last.role, agent::message::Role::User);
    assert_eq!(last.content, "解析失败");
}

// ===== 座位号宽容解析边界（走 VoteTool 契约）=====

#[test]
fn target_field_boundaries() {
    let ctx = context("投票", Role::Villager);
    let parse = |json: &str| {
        parse_tool_action(&ctx, &[call("vote", json)]).and_then(|a| match a {
            Action::Vote { target } => target,
            _ => None,
        })
    };
    // 字符串数字也认
    assert_eq!(parse(r#"{"target":"4"}"#), Some(4));
    // 死者 / 负数 / 非数字 → 校验不过 → 弃票
    assert_eq!(parse(r#"{"target":9}"#), None);
    assert_eq!(parse(r#"{"target":-1}"#), None);
    assert_eq!(parse(r#"{"target":"abc"}"#), None);
    // 缺字段 → serde default → 弃票
    assert_eq!(parse(r#"{}"#), None);
}

// ===== 夜间事件可见性（狼人不可知神职行动）=====

#[test]
fn night_actions_invisible_to_wolves_except_kill() {
    let guard = GameEvent::NightAction {
        day: 1,
        actor: PlayerRole { id: 1, role: Role::Guard },
        action: Action::GuardProtect { target: None },
    };
    let seer = GameEvent::NightAction {
        day: 1,
        actor: PlayerRole { id: 4, role: Role::Seer },
        action: Action::SeerCheck { target: 2 },
    };
    let kill = GameEvent::NightAction {
        day: 1,
        actor: PlayerRole { id: 3, role: Role::Werewolf },
        action: Action::WerewolfKill { target: 5 },
    };
    let chat = GameEvent::WolfChatSpoke {
        day: 1,
        speaker: PlayerRole { id: 3, role: Role::Werewolf },
        text: "刀5".into(),
    };
    // 狼人视角：守卫/预言家行动不可见，狼刀与狼聊可见
    assert!(!visible_to(&guard, true));
    assert!(!visible_to(&seer, true));
    assert!(visible_to(&kill, true));
    assert!(visible_to(&chat, true));
    // 好人视角：以上全部不可见
    assert!(!visible_to(&guard, false));
    assert!(!visible_to(&seer, false));
    assert!(!visible_to(&kill, false));
    assert!(!visible_to(&chat, false));
}
