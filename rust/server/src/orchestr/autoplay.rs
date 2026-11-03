//! 对局自动推进：引擎 pending → 选手 LLM 决策 → `apply` 喂回，循环到胜负已分。
//!
//! 决策上下文 = 身份知悉（含狼队友/预言家验人）+ 公开事件史（狼额外可见狼聊与夜刀）
//! + 玩家视角记忆（`orchestr::memory::recall`）+ 当前动作的 JSON 输出契约。
//!
//! LLM 不可用或解析失败时按规则兜底，保证局能推完；非法动作先回喂引擎拒绝原因重试一次。

//!
//! 锁纪律：`state.game.with_engine` 锁内只做同步快照收集，LLM 请求一律在锁外发起。

use std::time::Duration;

use agent::event::AgentEvent;
use agent::message::{ToolCall, ToolChoice, ToolDefinition};
use agent::{ChatMessage, ChatRequest, build_provider};
use macros::agent_tool;
use werewolf::{Action, CheckResult, GameEngine, GameEvent, Phase, PlayerId, Role};

use crate::routes::werewolf::{log_new_events, render_event};
use state::{AppState, GameStatus};

/// 相邻两步决策之间的停顿：给前端轮询留出渲染节奏，也压一压请求频率
const STEP_PAUSE: Duration = Duration::from_millis(400);
/// 单步决策的最大对话轮数（强制工具 → 纠错回喂 → 再纠错，超限交规则兜底）
const MAX_DECISION_TURNS: usize = 3;

/// 开局（或恢复）后启动自动推进任务；状态非 Running 或已结算时立即退出
pub fn spawn(state: &AppState, game_id: i64) {
    let state = state.clone();
    tokio::spawn(async move {
        drive(&state, game_id).await;
    });
}

/// 自动推进主循环（单局制）：引擎 settled 或控制面离开 Running 即结束
async fn drive(state: &AppState, game_id: i64) {
    loop {
        tokio::time::sleep(STEP_PAUSE).await;
        if !state
            .game
            .view(game_id)
            .is_ok_and(|view| view.status == GameStatus::Running)
        {
            return;
        }
        // 锁内：取 pending + 收集决策上下文快照（LLM 调用在锁外）。
        // actor 要先从 pending 拿到才能取记忆，两次短锁顺序获取不嵌套。
        let first = state
            .game
            .with_engine(game_id, |engine| {
                engine
                    .filter(|engine| !matches!(engine.phase(), Phase::Settled { .. }))
                    .and_then(GameEngine::pending)
            })
            .unwrap_or_default();
        let Some((actor, _kind)) = first else {
            continue;
        };
        let memory = state
            .game
            .with_memories(game_id, |memories| {
                memories
                    .get(&actor)
                    .map(memory::AgentMemory::render)
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        let context = state
            .game
            .with_engine(game_id, |engine| {
                engine.and_then(|engine| collect_context(engine, memory))
            })
            .unwrap_or_default();
        let Some(mut context) = context else {
            continue;
        };
        tracing::info!(actor, kind = context.kind, "autoplay 决策开始");
        // 座位 → 选手名（LLM 决策与配置解析都按名字）
        context.name = state
            .game
            .view(game_id)
            .ok()
            .and_then(|view| view.seats.get(actor as usize).cloned())
            .unwrap_or_else(|| format!("{actor}号选手"));
        step(state, game_id, actor, context).await;
        if state
            .game
            .with_engine(game_id, |engine| {
                engine.is_some_and(|e| matches!(e.phase(), Phase::Settled { .. }))
            })
            .unwrap_or(false)
        {
            return;
        }
    }
}

/// 一步决策：多轮工具对话（强制 `speak`/`vote`/… 工具调用）→ 解析 → apply。
///
/// 每轮结果（含引擎拒绝原因）都作为对话历史回喂，模型能看到自己上一轮说了什么；
/// 超限或 LLM 不可用时交规则兜底，保证局能推完
async fn step(state: &AppState, game_id: i64, actor: PlayerId, context: DecisionContext) {
    // 多轮历史：assistant(tool_calls) ↔ tool(results) 往返都在这里累积
    let mut history = vec![
        ChatMessage::system(system_prompt(&context)),
        ChatMessage::user(user_prompt(&context)),
    ];
    for _ in 0..MAX_DECISION_TURNS {
        let Some((text, calls)) = chat(state, &context, &history).await else {
            break;
        };
        // 先解析再入历史：纠错消息要与 assistant 调用配对（tool_calls 后必须跟 tool 消息）
        let action = parse_tool_action(&context, &calls).or_else(|| parse_action(&context, &text));
        // assistant 回复入历史（带 calls 时结果必须用 tool 消息回喂）
        if calls.is_empty() {
            history.push(ChatMessage::assistant(text.clone()));
        } else {
            history.push(ChatMessage::assistant_with_tools(text, calls.clone()));
        }
        let Some(action) = action else {
            push_correction(
                &mut history,
                &calls,
                "你的输出无法解析为游戏动作，请严格按工具参数填写后重新调用。",
            );
            continue;
        };
        match apply(state, game_id, actor, action) {
            // 已推进，本步结束
            None => return,
            Some(rejection) => push_correction(
                &mut history,
                &calls,
                &format!("游戏引擎拒绝了你的动作：{rejection}。请换一个合法动作。"),
            ),
        }
    }
    // LLM 不可用 / 始终无法解析 / 始终被拒：规则兜底，保证局能推完
    if let Some(fallback) = fallback_action(&context) {
        if apply(state, game_id, actor, fallback).is_some() {
            tracing::warn!(
                actor,
                kind = context.kind,
                "兜底动作也被引擎拒绝，等待下一轮"
            );
        }
    } else {
        tracing::warn!(actor, kind = context.kind, "无规则兜底，等待下一轮");
    }
}

/// 把纠错消息接进历史：有工具调用时用 tool 结果回喂（协议硬性配对），否则用 user 消息
fn push_correction(history: &mut Vec<ChatMessage>, calls: &[ToolCall], correction: &str) {
    match calls.first() {
        Some(call) => history.push(ChatMessage::tool_result(&call.id, correction)),
        None => history.push(ChatMessage::user(correction)),
    }
}

/// 引擎喂入动作；`Some(错误)` = 被拒绝（需要重试），`None` = 已推进
fn apply(state: &AppState, game_id: i64, actor: PlayerId, action: Action) -> Option<String> {
    state
        .game
        .with_engine_mut(game_id, |engine| {
            engine
                .ok_or_else(|| "对局已不存在".to_string())
                .and_then(|engine| {
                    let seen = engine.events().len();
                    engine
                        .apply(actor, action)
                        .map(|_| log_new_events(engine, seen))
                        .map_err(|err: werewolf::GameError| err.to_string())
                })
                .err()
        })
        .unwrap_or_default()
}

/// 一步决策所需的全部快照信息
struct DecisionContext {
    /// 座位号
    seat: PlayerId,
    /// 选手名（prompt 中的自称；也用于按名取模型配置）
    name: String,
    /// 本人身份
    role: Role,
    /// 狼队友座位（仅狼人非空）
    wolf_mates: Vec<PlayerId>,
    /// 存活座位
    alive: Vec<PlayerId>,
    /// 公开事件史（狼人视角额外含狼聊与夜刀）
    history: String,
    /// 预言家的私有验人记录 `(天, 目标, 结论)`（仅预言家非空）
    checks: Vec<(u32, PlayerId, CheckResult)>,
    /// 玩家视角记忆文本（可为空）
    memory: String,
    /// 当前动作提示（引擎 pending 的 kind）
    kind: &'static str,
    /// 当前天数
    day: u32,
}

/// 锁内收集一步决策的上下文（不做任何 IO）
fn collect_context(engine: &GameEngine, memory: String) -> Option<DecisionContext> {
    let players = engine.players();
    // 调用方（autoplay_step）已确认 pending 存在
    let (seat, kind) = engine.pending()?;
    let role = players[seat as usize].role;
    let is_wolf = role == Role::Werewolf;
    Some(DecisionContext {
        seat,
        name: String::new(),
        role,
        wolf_mates: if is_wolf {
            players
                .iter()
                .filter(|p| p.role == Role::Werewolf && p.id != seat && p.alive)
                .map(|p| p.id)
                .collect()
        } else {
            Vec::new()
        },
        alive: players.iter().filter(|p| p.alive).map(|p| p.id).collect(),
        history: engine
            .events()
            .iter()
            .filter(|event| visible_to(event, is_wolf))
            .filter_map(|event| render_event(event, is_wolf))
            .map(|(_, _, _, text)| text)
            .collect::<Vec<_>>()
            .join("\n"),
        checks: players[seat as usize]
            .checks
            .iter()
            .map(|c| (c.day, c.target, c.result))
            .collect(),
        memory,
        kind,
        day: engine.phase().day(),
    })
}

/// 事件对视角的可见性：狼人额外可见狼聊与狼刀；守卫/女巫/预言家等夜间行动
/// 对任何决策视角都不可见（对齐 snapshot.rs 的 Viewer 规则）
fn visible_to(event: &GameEvent, is_wolf: bool) -> bool {
    match event {
        GameEvent::WolfChatSpoke { .. } | GameEvent::WolfChatPassed { .. } => is_wolf,
        GameEvent::NightAction { action, .. } => {
            is_wolf && matches!(action, Action::WerewolfKill { .. })
        }
        _ => true,
    }
}

// ---------------------------------------------------------------------------
// LLM 调用（锁外）
// ---------------------------------------------------------------------------

/// 一轮决策对话：带工具定义（强制工具调用），返回（文本回复, 工具调用列表）。
///
/// 历史由调用方累积（多轮往返），本函数只负责发请求 + 收事件
async fn chat(
    state: &AppState,
    context: &DecisionContext,
    history: &[ChatMessage],
) -> Option<(String, Vec<ToolCall>)> {
    let global = state.agent.config();
    let config = state.roster.resolve(&context.name, &global);
    let provider = build_provider(&config).ok()?;
    let request = ChatRequest {
        model: None,
        messages: history.to_vec(),
        thinking: config.thinking,
        // 决策必须是结构化动作：一个工具 + Required 强制调用；
        // 网关不支持 tool_choice 时多数会降级为 auto，文本 JSON 契约仍能兜底
        tools: vec![tool_for(context)],
        tool_choice: ToolChoice::Required,
    };
    let mut stream = provider.chat_stream(request).await.ok()?;
    // 决策级超时：比 client 整体超时更快兜底，单轮 90s 内不出果就交下一轮/规则兜底
    let deadline = std::time::Duration::from_secs(90);
    let (reply, calls) = tokio::time::timeout(deadline, async {
        let mut reply = String::new();
        let mut calls = Vec::new();
        while let Ok(Some(event)) = stream.next_event().await {
            match event {
                AgentEvent::TextDelta { delta } => reply.push_str(&delta),
                AgentEvent::ToolCalls { calls: batch } => calls.extend(batch),
                _ => {}
            }
        }
        (reply, calls)
    })
    .await
    .ok()?;
    Some((reply, calls))
}

// ---------------------------------------------------------------------------
// Prompt
// ---------------------------------------------------------------------------

fn system_prompt(context: &DecisionContext) -> String {
    let role_text = context.role.display_name();
    let mut prompt = format!(
        "你是 AI 狼人杀选手「{name}」，坐在 {seat} 号位，身份是{role_text}。\n\
         玩法：夜晚狼人杀人，白天好人靠发言与投票放逐狼人；狼人要隐藏身份，好人要推理找狼。\n",
        name = context.name,
        seat = context.seat,
    );
    if context.role == Role::Werewolf {
        let mates = context
            .wolf_mates
            .iter()
            .map(|id| format!("{id}号"))
            .collect::<Vec<_>>()
            .join("、");
        prompt.push_str(&format!("你的狼队友：{mates}（夜间狼聊只有狼队可见）。\n"));
    }
    if !context.checks.is_empty() {
        let checks = context
            .checks
            .iter()
            .map(|(day, target, result)| format!("第{day}天验{target}号：{}", check_text(*result)))
            .collect::<Vec<_>>()
            .join("；");
        prompt.push_str(&format!("你的验人记录：{checks}。\n"));
    }
    prompt.push_str(
        "要求：每次只输出一个 JSON 对象，不要 markdown 代码块，不要解释。\n\
         发言要短（两三句以内），像真人玩家：狼人可以伪装悍跳，好人直接说推理。\n",
    );
    prompt
}

fn user_prompt(context: &DecisionContext) -> String {
    let alive = context
        .alive
        .iter()
        .map(|id| format!("{id}号"))
        .collect::<Vec<_>>()
        .join("、");
    let memory = if context.memory.is_empty() {
        "（暂无）".to_string()
    } else {
        context.memory.clone()
    };
    format!(
        "【局面】第{}天；存活：{}\n【事件记录】\n{}\n【你的记忆】\n{}\n\n【当前要求】{}\n{}",
        context.day,
        alive,
        context.history,
        memory,
        context.kind,
        action_contract(context.kind)
    )
}

/// 各动作的 JSON 输出契约（含例子）
fn action_contract(kind: &str) -> &'static str {
    match kind {
        "白天发言" => {
            r#"输出 JSON：{"text": "发言内容"}。例：{"text": "我是平民，昨天3号的发言很可疑，建议票他。"}"#
        }
        "投票" => {
            r#"输出 JSON：{"target": 座位号或null}。例：{"target": 3}；弃票 {"target": null}"#
        }
        "狼人夜聊" => {
            r#"输出 JSON：{"text": "耳语内容或null"}。例：{"text": "我先刀4号，你们别暴露"}；沉默 {"text": null}"#
        }
        "狼人行动" => {
            r#"输出 JSON：{"target": 要刀的座位号}。例：{"target": 4}。注意别刀自己队友"#
        }
        "夜晚行动" => {
            r#"输出 JSON。预言家：{"target": 要验的座位号}（必填，验出好人/狼人）；女巫：{"save": true或false, "target": 毒药目标或null}（save=是否用解药，target=毒药目标）；守卫：{"target": 守护座位号或null}。例：{"target": 5}"#
        }
        "骑士决斗" => r#"输出 JSON：{"target": 座位号或null}。决斗失败你会死，谨慎使用"#,
        "猎人开枪" => {
            r#"输出 JSON：{"target": 座位号或null}。例：{"target": 3}；放弃 {"target": null}"#
        }
        _ => r#"输出 JSON：{"target": 座位号或null}"#,
    }
}

fn check_text(result: CheckResult) -> &'static str {
    match result {
        CheckResult::Good => "好人",
        CheckResult::Werewolf => "狼人",
    }
}

// ---------------------------------------------------------------------------
// 回复解析（宽松取首个 JSON 对象）
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// 工具定义与工具调用解析（tool_calls 标准优先，文本 JSON 兜底）
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// 工具契约（#[agent_tool] 生成 definition/parse；场景映射与合法性校验留在下方）
// ---------------------------------------------------------------------------

/// 座位号宽容解析（可空目标）：数字或数字字符串均可（弱模型兜底）；缺失/null/非法 → None
fn lenient_target<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize as _;
    match Option::<serde_json::Value>::deserialize(deserializer)? {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(num)) => Ok(num.as_u64().and_then(|n| u8::try_from(n).ok())),
        Some(serde_json::Value::String(text)) => Ok(text.trim().parse().ok()),
        Some(_) => Ok(None),
    }
}

/// 座位号宽容解析（必填目标）：接受数字或数字字符串；缺失/null/非法 → 解析失败 → 纠错重问
fn lenient_required_target<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize as _;
    match Option::<serde_json::Value>::deserialize(deserializer)? {
        Some(serde_json::Value::Number(num)) => num
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .ok_or_else(|| serde::de::Error::custom("无效座位号")),
        Some(serde_json::Value::String(text)) => {
            text.trim().parse().map_err(serde::de::Error::custom)
        }
        _ => Err(serde::de::Error::custom("缺少座位号")),
    }
}

#[agent_tool(
    name = "speak",
    description = "提交你的白天发言。要 4~6 句（100~200 字），有具体推理依据：解读死讯、指出怀疑对象及理由、回应他人、表明投票意向，像真人口语"
)]
struct SpeakTool {
    #[param(description = "发言内容")]
    text: String,
}

#[agent_tool(name = "vote", description = "提交你的放逐投票。弃票把 target 留空")]
struct VoteTool {
    #[param(description = "要放逐的座位号，弃票留空")]
    #[serde(default, deserialize_with = "lenient_target")]
    target: Option<u8>,
}

#[agent_tool(
    name = "wolf_chat",
    description = "狼队夜间耳语：给队友传达刀口意向或身份判断。沉默把 text 留空"
)]
struct WolfChatTool {
    #[param(description = "耳语内容，沉默留空")]
    #[serde(default)]
    text: Option<String>,
}

#[agent_tool(
    name = "wolf_kill",
    description = "狼队今晚要刀的玩家。注意别刀自己队友"
)]
struct WolfKillTool {
    #[param(description = "要刀的座位号，不能选自己和队友")]
    #[serde(deserialize_with = "lenient_required_target")]
    target: u8,
}

#[agent_tool(name = "seer_check", description = "查验一名玩家的身份（好人/狼人）")]
struct SeerCheckTool {
    #[param(description = "要查验的座位号，不能选自己")]
    #[serde(deserialize_with = "lenient_required_target")]
    target: u8,
}

#[agent_tool(
    name = "witch_act",
    description = "使用药水：save=是否用解药救今晚刀口；target=毒药目标（可留空）。解药毒药各只能用一次，同晚不能同时用"
)]
struct WitchActTool {
    #[param(description = "是否使用解药")]
    save: bool,
    #[param(description = "毒药目标座位号，不毒留空")]
    #[serde(default, deserialize_with = "lenient_target")]
    target: Option<u8>,
}

#[agent_tool(
    name = "guard_protect",
    description = "守护一名玩家今晚免受狼人攻击。不能连续两晚守同一人，可不守"
)]
struct GuardProtectTool {
    #[param(description = "要守护的座位号，不守留空")]
    #[serde(default, deserialize_with = "lenient_target")]
    target: Option<u8>,
}

#[agent_tool(
    name = "knight_duel",
    description = "与一名玩家决斗：验出狼人则对方死，否则你死。谨慎使用，可放弃"
)]
struct KnightDuelTool {
    #[param(description = "决斗对象座位号，放弃留空")]
    #[serde(default, deserialize_with = "lenient_target")]
    target: Option<u8>,
}

#[agent_tool(
    name = "hunter_shot",
    description = "出局时开枪带走一名玩家。不确定目标可放弃，误伤好人会亏"
)]
struct HunterShotTool {
    #[param(description = "开枪目标座位号，放弃留空")]
    #[serde(default, deserialize_with = "lenient_target")]
    target: Option<u8>,
}

/// 当前决策场景的工具契约（场景/角色 → 工具的映射是业务决策，保留显式 match）
fn tool_for(context: &DecisionContext) -> ToolDefinition {
    let night = match context.role {
        Role::Seer => SeerCheckTool::definition(),
        Role::Witch => WitchActTool::definition(),
        Role::Guard => GuardProtectTool::definition(),
        _ => WolfKillTool::definition(),
    };
    match context.kind {
        "白天发言" => SpeakTool::definition(),
        "投票" => VoteTool::definition(),
        "狼人夜聊" => WolfChatTool::definition(),
        "狼人行动" => WolfKillTool::definition(),
        "夜晚行动" => night,
        "骑士决斗" => KnightDuelTool::definition(),
        "猎人开枪" => HunterShotTool::definition(),
        _ => VoteTool::definition(),
    }
}

/// 工具参数/文本 JSON → 动作。工具调用与文本兜底两种载体同构
/// （{"text":…}/{"target":…}），共用契约 struct 解析；
/// 身份合法性（存活/非自己）在这里统一校验
fn action_from_args(context: &DecisionContext, tool_name: &str, arguments: &str) -> Option<Action> {
    let alive_target =
        |target: Option<u8>| target.filter(|id| *id != context.seat && context.alive.contains(id));
    match tool_name {
        "speak" => {
            let tool = SpeakTool::parse(arguments)?;
            let text = tool.text.trim().to_string();
            (!text.is_empty()).then_some(Action::Speech { text })
        }
        "vote" => VoteTool::parse(arguments).map(|tool| Action::Vote {
            target: alive_target(tool.target),
        }),
        "wolf_chat" => WolfChatTool::parse(arguments).map(|tool| Action::WolfChat {
            text: tool.text.filter(|t| !t.trim().is_empty()),
        }),
        "wolf_kill" => WolfKillTool::parse(arguments)
            .map(|tool| alive_target(Some(tool.target)))
            .and_then(|target| target.map(|target| Action::WerewolfKill { target })),
        "seer_check" => SeerCheckTool::parse(arguments)
            .map(|tool| alive_target(Some(tool.target)))
            .and_then(|target| target.map(|target| Action::SeerCheck { target })),
        "witch_act" => WitchActTool::parse(arguments).map(|tool| Action::WitchAct {
            save: tool.save,
            poison: alive_target(tool.target),
        }),
        "guard_protect" => GuardProtectTool::parse(arguments).map(|tool| Action::GuardProtect {
            target: alive_target(tool.target),
        }),
        "knight_duel" => KnightDuelTool::parse(arguments).map(|tool| Action::KnightDuel {
            target: alive_target(tool.target),
        }),
        "hunter_shot" => HunterShotTool::parse(arguments).map(|tool| Action::HunterShot {
            target: alive_target(tool.target),
        }),
        _ => None,
    }
}

/// 优先从模型的 tool_calls 解析动作；无匹配工具或参数非法返回 `None`（纠错重问兜底）
fn parse_tool_action(context: &DecisionContext, calls: &[ToolCall]) -> Option<Action> {
    let expected = tool_for(context).name;
    let call = calls.iter().find(|call| call.name == expected)?;
    action_from_args(context, expected.as_str(), &call.arguments)
}

/// 从 LLM 文本回复解析动作（工具调用失败时的兜底；与工具参数同构，复用同一套契约）
fn parse_action(context: &DecisionContext, reply: &str) -> Option<Action> {
    let object = first_json_object(reply)?;
    let expected = tool_for(context).name;
    serde_json::to_string(&object)
        .ok()
        .and_then(|args| action_from_args(context, expected.as_str(), &args))
}

/// 从 LLM 回复宽松解析首个 `{...}` JSON 对象（mvp 投票复用）
pub(crate) fn first_json_object(reply: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    let start = reply.find('{')?;
    let end = reply[start..].find('}')? + start;
    serde_json::from_str::<serde_json::Value>(&reply[start..=end])
        .ok()?
        .as_object()
        .cloned()
}

// ---------------------------------------------------------------------------
// 规则兜底（LLM 不可用 / 解析失败 / 始终被拒）
// ---------------------------------------------------------------------------

/// 兜底动作：可空动作一律选"放弃"，必须给目标的按简单启发式选
fn fallback_action(context: &DecisionContext) -> Option<Action> {
    let others: Vec<PlayerId> = context
        .alive
        .iter()
        .copied()
        .filter(|id| *id != context.seat)
        .collect();
    match context.kind {
        // 兜底发言不提身份（旧文案「我是狼人，过」会自爆）
        "白天发言" => Some(Action::Speech {
            text:
                "这轮我先听大家的推理，目前我倾向跟票发言最有破绽的人，后面有更硬的逻辑我再改票。"
                    .to_string(),
        }),
        "投票" => Some(Action::Vote {
            target: others.first().copied(),
        }),
        "狼人夜聊" => Some(Action::WolfChat { text: None }),
        "狼人行动" => {
            // 狼不刀队友
            let wolves = context.wolf_mates.iter().copied().chain([context.seat]);
            let wolves: Vec<PlayerId> = wolves.collect();
            Some(Action::WerewolfKill {
                target: others.into_iter().find(|id| !wolves.contains(id))?,
            })
        }
        "夜晚行动" => match context.role {
            Role::Witch => Some(Action::WitchAct {
                save: false,
                poison: None,
            }),
            Role::Guard => Some(Action::GuardProtect { target: None }),
            // 预言家必须验人：验第一个未验过的活人
            Role::Seer => {
                let checked: Vec<PlayerId> = context.checks.iter().map(|(_, t, _)| *t).collect();
                Some(Action::SeerCheck {
                    target: others.into_iter().find(|id| !checked.contains(id))?,
                })
            }
            _ => None,
        },
        "骑士决斗" => Some(Action::KnightDuel { target: None }),
        "猎人开枪" => Some(Action::HunterShot { target: None }),
        _ => None,
    }
}
#[cfg(test)]
#[path = "autoplay_tests.rs"]
mod tests;
