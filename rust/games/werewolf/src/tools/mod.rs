use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use ::tools::{Args, Tool, ToolError, ToolRegistry};

use crate::action::Action;
use crate::engine::GameEngine;
use crate::phase::PlayerId;
use crate::role::Role;

mod window;

pub use window::{
    GuardProtectTool, HunterShotTool, KnightDuelTool, SeerCheckTool, SpeechTool, VoteTool,
    WerewolfChatTool, WerewolfKillTool, WitchActTool,
};

/// 工具与引擎的共享句柄：多个工具持有同一份对局，锁内串行推进。
///
/// 工具自身无状态（`Send + Sync`），谁持有 [`EngineHandle`]
/// 谁就能以任意角色的窗口喂动作——越权 / 阶段不符由引擎校验兜底。
#[derive(Clone)]
pub struct EngineHandle(Arc<Mutex<GameEngine>>);

impl EngineHandle {
    /// 包住一局引擎
    pub fn new(engine: GameEngine) -> Self {
        Self(Arc::new(Mutex::new(engine)))
    }

    /// 锁内推进一次：取当前窗口的行动者，喂入动作，返回增量事件。
    ///
    /// 出参约定（工具统一返回形状，方便上层直接透传给 LLM）：
    /// `actor` 行动者座位 / `window` 窗口名 / `action` 动作描述 / `events` 增量事件
    pub fn run(&self, tool: &str, action: Action) -> Result<Value, ToolError> {
        let mut engine = self
            .0
            .lock()
            .map_err(|_| ToolError::execution(tool, "引擎锁中毒，对局状态不可恢复"))?;
        let Some((actor, window)) = engine.pending() else {
            return Err(ToolError::execution(tool, "对局已结束，没有待行动窗口"));
        };
        let events = engine
            .apply(actor, action.clone())
            .map_err(|err| ToolError::execution(tool, err.to_string()))?;
        Ok(json!({
            "actor": actor,
            "window": window,
            "action": action.describe(),
            "events": events,
        }))
    }

    /// 当前存活玩家座位数（编排层排版用）
    pub fn total_players(&self) -> u8 {
        self.0.lock().map_or(0, |engine| engine.total_players())
    }

    /// 上帝视角查角色座位（编排层用它决定该提示哪个 agent；取存活者）
    pub fn seat_of(&self, role: Role) -> Option<PlayerId> {
        let engine = self.0.lock().ok()?;
        engine
            .players()
            .iter()
            .find(|p| p.role == role && p.alive)
            .map(|p| p.id)
    }
}

/// 可选目标座位参数 → `Option<PlayerId>`：缺省 / `null` 即「不指定」（空守 / 弃票等）
fn opt_target(tool: &str, args: &Args, field: &str) -> Result<Option<u8>, ToolError> {
    args.opt_u8(field)
        .map_err(|_| ToolError::invalid_field(tool, field, "0..=255 的座位号"))
}

/// 目标座位参数 → `PlayerId`：必填
fn req_target(tool: &str, args: &Args) -> Result<u8, ToolError> {
    args.u8("target")
        .map_err(|_| ToolError::invalid_field(tool, "target", "0..=255 的座位号"))
}

fn target_schema(required: bool) -> Value {
    let mut schema = json!({
        "type": "object",
        "properties": {
            "target": { "type": "integer", "minimum": 0, "description": "目标座位号" }
        }
    });
    if required {
        schema["required"] = json!(["target"]);
    }
    schema
}

/// 组装本局默认工具注册表：按夜晚 → 白天行动顺序注册全部窗口工具。
///
/// 说明书顺序即 prompt 顺序，注册顺序保持稳定
pub fn registry(engine: EngineHandle) -> Result<ToolRegistry, ToolError> {
    let mut registry = ToolRegistry::new();
    for tool in [
        Box::new(GuardProtectTool::new(engine.clone())) as Box<dyn Tool>,
        Box::new(WerewolfChatTool::new(engine.clone())),
        Box::new(WerewolfKillTool::new(engine.clone())),
        Box::new(WitchActTool::new(engine.clone())),
        Box::new(SeerCheckTool::new(engine.clone())),
        Box::new(SpeechTool::new(engine.clone())),
        Box::new(KnightDuelTool::new(engine.clone())),
        Box::new(VoteTool::new(engine.clone())),
        Box::new(HunterShotTool::new(engine)),
    ] {
        registry.register(tool)?;
    }
    Ok(registry)
}
