use serde_json::{Value, json};

use ::tools::{Args, Tool, ToolDefinition, ToolError};

use super::{EngineHandle, opt_target};

/// 狼队夜聊耳语：当前游标狼发言或保持沉默，说完自动轮到下一位狼
pub struct WerewolfChatTool {
    engine: EngineHandle,
}

impl WerewolfChatTool {
    pub fn new(engine: EngineHandle) -> Self {
        Self { engine }
    }
}

impl Tool for WerewolfChatTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "wolf_chat",
            "狼队夜聊：以当前狼的身份说一句耳语（仅狼队可见）；不带 text 表示本轮保持沉默。说完自动轮到下一位狼",
        )
        .parameters(json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "耳语内容（战术沟通 / 报刀口意图）" }
            }
        }))
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let args = Args::parse("wolf_chat", args)?;
        let text = args.opt_string("text")?.filter(|t| !t.is_empty());
        self.engine
            .run("wolf_chat", crate::Action::WolfChat { text })
    }
}

/// 狼队共同刀一人（多狼共享一次决定；进入刀人窗口后任意存活狼可提交）
pub struct WerewolfKillTool {
    engine: EngineHandle,
}

impl WerewolfKillTool {
    pub fn new(engine: EngineHandle) -> Self {
        Self { engine }
    }
}

impl Tool for WerewolfKillTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "werewolf_kill",
            "狼人刀人：狼队共同袭击一名玩家（目标必须存活）。进入刀人窗口后由当前窗口的狼提交",
        )
        .parameters(super::target_schema(true))
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let args = Args::parse("werewolf_kill", args)?;
        let target = super::req_target("werewolf_kill", &args)?;
        self.engine
            .run("werewolf_kill", crate::Action::WerewolfKill { target })
    }
}

/// 女巫行动：是否用解药救当晚刀口 + 是否毒人；同晚至多用一瓶
pub struct WitchActTool {
    engine: EngineHandle,
}

impl WitchActTool {
    pub fn new(engine: EngineHandle) -> Self {
        Self { engine }
    }
}

impl Tool for WitchActTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "witch_act",
            "女巫行动：决定是否用解药救下今晚的刀口（save）以及是否毒人（poison）。同晚至多用一瓶药；两瓶都用完则只能按兵不动（save=false 且不带 poison）",
        )
        .parameters(json!({
            "type": "object",
            "properties": {
                "save": { "type": "boolean", "description": "是否用解药救今晚刀口" },
                "poison": { "type": "integer", "minimum": 0, "description": "毒杀目标座位号；不带或 null 表示今晚不毒人" }
            },
            "required": ["save"]
        }))
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let args = Args::parse("witch_act", args)?;
        let save = args.boolean("save")?;
        let poison = opt_target("witch_act", &args, "poison")?;
        self.engine
            .run("witch_act", crate::Action::WitchAct { save, poison })
    }
}

/// 预言家验人：查验一名存活玩家（不能验自己）的阵营
pub struct SeerCheckTool {
    engine: EngineHandle,
}

impl SeerCheckTool {
    pub fn new(engine: EngineHandle) -> Self {
        Self { engine }
    }
}

impl Tool for SeerCheckTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "seer_check",
            "预言家查验：查验一名存活玩家的阵营（好人 / 狼人）。不能验自己；结果记入私有验人记录",
        )
        .parameters(super::target_schema(true))
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let args = Args::parse("seer_check", args)?;
        let target = super::req_target("seer_check", &args)?;
        self.engine
            .run("seer_check", crate::Action::SeerCheck { target })
    }
}

/// 守卫守护：守护一名玩家（可空守）；不能连续两晚守同一人
pub struct GuardProtectTool {
    engine: EngineHandle,
}

impl GuardProtectTool {
    pub fn new(engine: EngineHandle) -> Self {
        Self { engine }
    }
}

impl Tool for GuardProtectTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "guard_protect",
            "守卫守护：今晚守护一名玩家使其免受狼刀（被女巫救则奶穿死亡）。不能连续两晚守同一人；不带 target 表示空守",
        )
        .parameters(super::target_schema(false))
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let args = Args::parse("guard_protect", args)?;
        let target = opt_target("guard_protect", &args, "target")?;
        self.engine
            .run("guard_protect", crate::Action::GuardProtect { target })
    }
}

/// 白天发言：当前发言者按座位序发言
pub struct SpeechTool {
    engine: EngineHandle,
}

impl SpeechTool {
    pub fn new(engine: EngineHandle) -> Self {
        Self { engine }
    }
}

impl Tool for SpeechTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "speech",
            "白天发言：以当前发言者身份发表一段对局发言（非空）。发言者由引擎按座位序轮转，无需（也不能）指定",
        )
        .parameters(json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "发言内容" }
            },
            "required": ["text"]
        }))
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let args = Args::parse("speech", args)?;
        let text = args.string("text")?;
        self.engine.run("speech", crate::Action::Speech { text })
    }
}

/// 投票：投一名存活玩家（可弃票）
pub struct VoteTool {
    engine: EngineHandle,
}

impl VoteTool {
    pub fn new(engine: EngineHandle) -> Self {
        Self { engine }
    }
}

impl Tool for VoteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "vote",
            "白天投票：把放逐票投给一名存活玩家；不带 target 表示弃票。每名有投票权的玩家依次投一票",
        )
        .parameters(super::target_schema(false))
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let args = Args::parse("vote", args)?;
        let target = opt_target("vote", &args, "target")?;
        self.engine.run("vote", crate::Action::Vote { target })
    }
}

/// 猎人开枪：出局时带走一人（可放弃；被毒不能开枪）
pub struct HunterShotTool {
    engine: EngineHandle,
}

impl HunterShotTool {
    pub fn new(engine: EngineHandle) -> Self {
        Self { engine }
    }
}

impl Tool for HunterShotTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "hunter_shot",
            "猎人开枪：猎人出局时开枪带走一名玩家（被毒杀时无法开枪）；不带 target 表示放弃开枪",
        )
        .parameters(super::target_schema(false))
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let args = Args::parse("hunter_shot", args)?;
        let target = opt_target("hunter_shot", &args, "target")?;
        self.engine
            .run("hunter_shot", crate::Action::HunterShot { target })
    }
}

/// 骑士决斗：白天翻牌决斗一名玩家（可放弃；一局一次）
pub struct KnightDuelTool {
    engine: EngineHandle,
}

impl KnightDuelTool {
    pub fn new(engine: EngineHandle) -> Self {
        Self { engine }
    }
}

impl Tool for KnightDuelTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "knight_duel",
            "骑士决斗：翻牌与一名玩家决斗——对方是狼人则狼死，否则骑士自尽；一局只能发动一次。不带 target 表示放弃决斗，进入投票",
        )
        .parameters(super::target_schema(false))
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let args = Args::parse("knight_duel", args)?;
        let target = opt_target("knight_duel", &args, "target")?;
        self.engine
            .run("knight_duel", crate::Action::KnightDuel { target })
    }
}
