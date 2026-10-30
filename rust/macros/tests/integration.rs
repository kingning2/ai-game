//! 集成测试：宏生成代码在真实 agent 类型上可用。
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use agent::ToolDefinition;
use macros::{agent_tool, game_event};

// ---------------------------------------------------------------------------
// agent_tool：合法用法
// ---------------------------------------------------------------------------

#[agent_tool(name = "seer_check", description = "查验一名玩家的身份")]
struct SeerCheck {
    #[param(description = "目标座位号（1-12，不能选自己）")]
    target: u8,
}

#[agent_tool(name = "witch_act", description = "使用药水")]
struct WitchAct {
    save: bool,
    #[param(description = "毒药目标，可空")]
    target: Option<u8>,
}

#[agent_tool(name = "speak", description = "提交发言")]
struct Speak {
    text: String,
    #[param(description = "引用的座位号列表")]
    references: Vec<u8>,
    #[param(description = "备注")]
    note: Option<String>,
}

#[test]
fn definition_generates_schema() {
    let def = SeerCheck::definition();
    assert_eq!(def.name, "seer_check");
    assert_eq!(def.description, "查验一名玩家的身份");

    let schema = def.parameters;
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"]["target"]["type"], "integer");
    assert_eq!(
        schema["properties"]["target"]["description"],
        "目标座位号（1-12，不能选自己）"
    );
    assert_eq!(schema["required"], serde_json::json!(["target"]));
}

#[test]
fn definition_handles_option_and_vec() {
    let schema = WitchAct::definition().parameters;
    assert_eq!(schema["properties"]["save"]["type"], "boolean");
    assert_eq!(schema["properties"]["target"]["type"], "integer");
    // Option 字段不进 required
    assert_eq!(schema["required"], serde_json::json!(["save"]));

    let schema = Speak::definition().parameters;
    assert_eq!(schema["properties"]["text"]["type"], "string");
    assert_eq!(schema["properties"]["references"]["type"], "array");
    assert_eq!(
        schema["properties"]["references"]["items"]["type"],
        "integer"
    );
    assert_eq!(schema["properties"]["note"]["type"], "string");
    assert_eq!(
        schema["required"],
        serde_json::json!(["text", "references"])
    );
}

#[test]
fn parse_roundtrip() {
    let parsed = SeerCheck::parse(r#"{"target": 5}"#).expect("合法参数应解析成功");
    assert_eq!(parsed.target, 5);

    let parsed = Speak::parse(r#"{"text":"发言","references":[3,5],"note":null}"#)
        .expect("全字段参数应解析成功");
    assert_eq!(parsed.references, vec![3, 5]);
    assert_eq!(parsed.note, None);
    assert_eq!(parsed.text, "发言");

    assert!(SeerCheck::parse(r#"{"target": "abc"}"#).is_none());
    assert!(SeerCheck::parse("not json").is_none());
    assert!(SeerCheck::parse("{}").is_none(), "缺必填字段应失败");

    // Option 缺省合法
    let parsed = WitchAct::parse(r#"{"save": true}"#).expect("Option 缺省应可解析");
    assert!(parsed.save && parsed.target.is_none());
}

// ---------------------------------------------------------------------------
// game_event：合法用法
// ---------------------------------------------------------------------------

#[game_event]
#[derive(Debug, Clone, PartialEq)]
enum DemoEvent {
    NightBegin { day: u32 },
    SpeechEnded { day: u32, seat: u8 },
    GameOver,
}

#[test]
fn game_event_day_dispatch() {
    assert_eq!(
        DemoEvent::NightBegin { day: 2 }.day_of(),
        Some(2),
        "含 day 字段的变体应提取天数"
    );
    assert_eq!(DemoEvent::SpeechEnded { day: 3, seat: 5 }.day_of(), Some(3));
    assert_eq!(DemoEvent::GameOver.day_of(), None, "无 day 字段归 None");
}

#[test]
fn game_event_preserves_serde() {
    // 宏不触碰原 enum：serde tag 序列化不受影响
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(tag = "type")]
    enum Tagged {
        Vote { day: u32, seat: u8 },
    }
    #[game_event]
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(tag = "type")]
    enum Tagged2 {
        Vote { day: u32, seat: u8 },
    }
    let json = serde_json::to_string(&Tagged::Vote { day: 1, seat: 2 }).expect("序列化");
    let back: Tagged2 = serde_json::from_str(&json).expect("反序列化");
    assert_eq!(back.day_of(), Some(1));
}

// ---------------------------------------------------------------------------
// definition() 与 ToolDefinition 的真实集成（OpenAI/Anthropic 双协议消费）
// ---------------------------------------------------------------------------

#[test]
fn definition_feeds_tool_definition() {
    let def: ToolDefinition = WitchAct::definition();
    // OpenAI 序列化格式：包一层 function
    let json = serde_json::to_value(&def).expect("ToolDefinition 可序列化");
    assert_eq!(json["name"], "witch_act");
    assert!(json["parameters"]["properties"]["save"].is_object());
}
