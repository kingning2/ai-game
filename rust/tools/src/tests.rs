use serde_json::{Value, json};

use crate::{Args, Tool, ToolDefinition, ToolError, ToolRegistry};

struct EchoTool;

impl Tool for EchoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("echo", "原样返回输入文本").parameters(json!({
            "type": "object",
            "properties": { "text": { "type": "string" } },
            "required": ["text"]
        }))
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let args = Args::parse("echo", args)?;
        Ok(json!({ "echo": args.string("text")? }))
    }
}

struct FailTool;

impl Tool for FailTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("fail", "永远报领域错误")
    }

    fn execute(&self, _args: Value) -> Result<Value, ToolError> {
        Err(ToolError::execution("fail", "领域规则拒绝"))
    }
}

#[test]
fn definition_defaults_and_builder() {
    let def = ToolDefinition::new("t", "说明");
    assert_eq!(
        def.parameters,
        json!({ "type": "object", "properties": {} })
    );
    let def = def.parameters(json!({ "type": "object" }));
    assert_eq!(def.parameters, json!({ "type": "object" }));
}

#[test]
fn registry_exports_definitions_in_order() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(EchoTool)).unwrap();
    registry.register(Box::new(FailTool)).unwrap();

    let names: Vec<_> = registry.definitions().into_iter().map(|d| d.name).collect();
    assert_eq!(names, vec!["echo", "fail"]);
}

#[test]
fn registry_rejects_duplicate_names() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(EchoTool)).unwrap();
    assert!(matches!(
        registry.register(Box::new(EchoTool)),
        Err(ToolError::DuplicateTool { name }) if name == "echo"
    ));
}

#[test]
fn registry_dispatches_by_name() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(EchoTool)).unwrap();

    let out = registry.execute("echo", json!({ "text": "你好" })).unwrap();
    assert_eq!(out, json!({ "echo": "你好" }));

    assert!(matches!(
        registry.execute("nope", json!({})),
        Err(ToolError::UnknownTool { name }) if name == "nope"
    ));
}

#[test]
fn args_type_checks() {
    let args = Args::parse("echo", json!({ "n": 3, "s": "hi", "b": true })).unwrap();

    assert_eq!(args.u8("n").unwrap(), 3);
    assert_eq!(args.opt_u8("missing").unwrap(), None);
    assert_eq!(args.string("s").unwrap(), "hi");
    assert_eq!(args.opt_string("missing").unwrap(), None);
    assert!(args.boolean("b").unwrap());
    assert!(args.opt_boolean("missing").unwrap().is_none());

    // 类型不对
    assert!(args.u8("s").is_err());
    assert!(args.string("n").is_err());
    assert!(args.boolean("s").is_err());
    // 超出 u8
    let big = Args::parse("echo", json!({ "n": 256 })).unwrap();
    assert!(big.opt_u8("n").is_err());
}

#[test]
fn args_missing_required_field() {
    let args = Args::parse("echo", json!({})).unwrap();
    assert!(matches!(
        args.u8("target"),
        Err(ToolError::InvalidArguments { tool, .. }) if tool == "echo"
    ));
}

#[test]
fn args_non_object_rejected() {
    assert!(Args::parse("echo", json!([1, 2])).is_err());
}

#[test]
fn execution_error_carries_domain_message() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(FailTool)).unwrap();
    assert!(matches!(
        registry.execute("fail", json!({})),
        Err(ToolError::Execution { tool, message }) if tool == "fail" && message == "领域规则拒绝"
    ));
}
