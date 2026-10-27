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
use agent::{ChatMessage, ChatRequest, build_provider};
use werewolf::{Action, CheckResult, GameEngine, GameEvent, Phase, PlayerId, Role};

use crate::routes::werewolf::render_event;
use state::{AppState, GameStatus};

/// 相邻两步决策之间的停顿：给前端轮询留出渲染节奏，也压一压请求频率
const STEP_PAUSE: Duration = Duration::from_millis(400);
/// 喂入非法动作被引擎拒绝后，把局面重新问一轮 LLM 的重试次数
const APPLY_RETRIES: usize = 1;

/// 开局（或恢复）后启动自动推进任务；状态非 Running 或已结算时立即退出
pub fn spawn(state: &AppState) {
    let state = state.clone();
    tokio::spawn(async move {
        drive(&state).await;
    });
}

/// 自动推进主循环（单局制）：引擎 settled 或控制面离开 Running 即结束
async fn drive(state: &AppState) {
    loop {
        tokio::time::sleep(STEP_PAUSE).await;
        if state.game.view().status != GameStatus::Running {
            return;
        }
        // 锁内：取 pending + 收集决策上下文快照（LLM 调用在锁外）。
        // actor 要先从 pending 拿到才能取记忆，两次短锁顺序获取不嵌套。
        let first = state.game.with_engine(|engine| {
            engine
                .filter(|engine| !matches!(engine.phase(), Phase::Settled { .. }))
                .and_then(GameEngine::pending)
        });
        let Some((actor, _kind)) = first else {
            continue;
        };
        let memory = state.game.with_memories(|memories| {
            memories
                .get(&actor)
                .map(autoplay_memory_text)
                .unwrap_or_default()
        });
        let context = state
            .game
            .with_engine(|engine| engine.and_then(|engine| collect_context(engine, memory)));
        let Some(mut context) = context else {
            continue;
        };
        // 座位 → 选手名（LLM 决策与配置解析都按名字）
        context.name = state
            .game
            .view()
            .seats
            .get(actor as usize)
            .cloned()
            .unwrap_or_else(|| format!("{actor}号选手"));
        step(state, actor, context).await;
        if state.game.with_engine(|engine| {
            engine.is_some_and(|e| matches!(e.phase(), Phase::Settled { .. }))
        }) {
            return;
        }
    }
}

/// 一步决策：prompt → LLM（锁外）→ 解析 → apply（被拒回喂重试 / 兜底）
async fn step(state: &AppState, actor: PlayerId, context: DecisionContext) {
    let mut reply = chat(state, &context, None).await;
    for _ in 0..=APPLY_RETRIES {
        let Some(action) = reply
            .as_deref()
            .and_then(|text| parse_action(&context, text))
        else {
            break;
        };
        let rejection = apply(state, actor, action);
        if rejection.is_none() {
            return;
        }
        // 引擎拒绝：把原因回喂再问一轮
        reply = chat(state, &context, rejection.as_deref()).await;
    }
    // LLM 不可用 / 解析失败 / 始终被拒：规则兜底，保证局能推完
    if let Some(fallback) = fallback_action(&context) {
        let _ = apply(state, actor, fallback);
    }
}

/// 引擎喂入动作；`Some(错误)` = 被拒绝（需要重试），`None` = 已推进
fn apply(state: &AppState, actor: PlayerId, action: Action) -> Option<String> {
    state.game.with_engine_mut(|engine| {
        engine
            .ok_or_else(|| "对局不存在".to_string())
            .and_then(|engine| {
                engine
                    .apply(actor, action)
                    .map(|_| ())
                    .map_err(|err: werewolf::GameError| err.to_string())
            })
            .err()
    })
}

// ---------------------------------------------------------------------------
// 决策上下文（锁内收集的快照）
// ---------------------------------------------------------------------------

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

/// 事件对视角的可见性：狼人额外可见狼聊与狼刀；夜晚神职行动对决策层不可见
fn visible_to(event: &GameEvent, is_wolf: bool) -> bool {
    match event {
        GameEvent::WolfChatSpoke { .. } | GameEvent::WolfChatPassed { .. } => is_wolf,
        GameEvent::NightAction { .. } => is_wolf,
        _ => true,
    }
}

// ---------------------------------------------------------------------------
// LLM 调用（锁外）
// ---------------------------------------------------------------------------

/// 单轮决策对话。`rejection` 非空时作为追加纠错消息（引擎拒绝原因回喂）。
async fn chat(
    state: &AppState,
    context: &DecisionContext,
    rejection: Option<&str>,
) -> Option<String> {
    let global = state.agent.config();
    let config = state.roster.resolve(&context.name, &global);
    let provider = build_provider(&config).ok()?;
    let mut messages = vec![
        ChatMessage::system(system_prompt(context)),
        ChatMessage::user(user_prompt(context)),
    ];
    if let Some(rejection) = rejection {
        messages.push(ChatMessage::user(format!(
            "你的上一个输出被游戏引擎拒绝了：{rejection}。请重新只输出一个合法 JSON。"
        )));
    }
    let request = ChatRequest {
        model: None,
        messages,
        thinking: config.thinking,
    };
    let mut stream = provider.chat_stream(request).await.ok()?;
    let mut reply = String::new();
    while let Ok(Some(event)) = stream.next_event().await {
        if let AgentEvent::TextDelta { delta } = event {
            reply.push_str(&delta);
        }
    }
    Some(reply)
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
            .map(|(day, target, result)| format!("第{day}天验{target}号：{}", check_text(result)))
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
        "夜晚行动" => r#"输出 JSON：{"target": 座位号或null}（按你的身份决定目标）"#,
        "骑士决斗" => r#"输出 JSON：{"target": 座位号或null}。决斗失败你会死，谨慎使用"#,
        "猎人开枪" => {
            r#"输出 JSON：{"target": 座位号或null}。例：{"target": 3}；放弃 {"target": null}"#
        }
        _ => r#"输出 JSON：{"target": 座位号或null}"#,
    }
}

fn check_text(result: &CheckResult) -> &'static str {
    match result {
        CheckResult::Good => "好人",
        CheckResult::Werewolf => "狼人",
    }
}

// ---------------------------------------------------------------------------
// 回复解析（宽松取首个 JSON 对象）
// ---------------------------------------------------------------------------

/// 从 LLM 回复解析动作；失败返回 `None`（由调用方兜底）
fn parse_action(context: &DecisionContext, reply: &str) -> Option<Action> {
    let object = first_json_object(reply)?;
    match context.kind {
        "白天发言" => {
            let text = string_field(object, "text")?;
            (!text.trim().is_empty()).then_some(Action::Speech { text })
        }
        "投票" => Some(Action::Vote {
            target: target_field(object, context),
        }),
        "狼人夜聊" => Some(Action::WolfChat {
            text: string_field(object, "text").filter(|t| !t.trim().is_empty()),
        }),
        "狼人行动" => Some(Action::WerewolfKill {
            target: target_field(object, context)?,
        }),
        "夜晚行动" => night_action(context, object),
        "骑士决斗" => Some(Action::KnightDuel {
            target: target_field(object, context),
        }),
        "猎人开枪" => Some(Action::HunterShot {
            target: target_field(object, context),
        }),
        _ => None,
    }
}

/// 夜晚行动按身份分派：女巫是 `{"save","poison"}`，其余是 `{"target"}`
fn night_action(context: &DecisionContext, object: &str) -> Option<Action> {
    match context.role {
        Role::Witch => {
            let save = object.contains("\"save\": true") || object.contains("\"save\":true");
            let poison = target_field(object, context);
            Some(Action::WitchAct { save, poison })
        }
        Role::Guard => Some(Action::GuardProtect {
            target: target_field(object, context),
        }),
        Role::Seer => Some(Action::SeerCheck {
            target: target_field(object, context)?,
        }),
        _ => None,
    }
}

/// 宽松提取回复中的首个 `{...}` JSON 对象
fn first_json_object(reply: &str) -> Option<&str> {
    let start = reply.find('{')?;
    let end = reply[start..].find('}')? + start;
    Some(&reply[start..=end])
}

fn string_field(object: &str, key: &str) -> Option<String> {
    let mut cursor = 0;
    while let Some(pos) = object[cursor..].find(&format!("\"{key}\"")) {
        let rest = &object[cursor + pos..];
        let colon = rest.find(':')?;
        let tail = &rest[colon + 1..];
        if let Some(value) = json_string_at(tail) {
            return Some(value);
        }
        cursor += pos + key.len() + 2;
    }
    None
}

/// 从 `tail`（已越过冒号）解析 JSON 字符串值（处理转义）
fn json_string_at(tail: &str) -> Option<String> {
    let quote = tail.find('"')?;
    let mut value = String::new();
    let mut chars = tail[quote + 1..].chars();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Some(value),
            '\\' => match chars.next()? {
                'n' => value.push('\n'),
                't' => value.push('\t'),
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    let code = u32::from_str_radix(&hex, 16).ok()?;
                    value.push(char::from_u32(code)?);
                }
                escaped => value.push(escaped),
            },
            other => value.push(other),
        }
    }
    None
}

/// 解析座位号字段；缺失 / null / 越界 / 指向死者 → `None`（调用方决定是否可空）
fn target_field(object: &str, context: &DecisionContext) -> Option<PlayerId> {
    let key_pos = object.find("\"target\"")?;
    let tail = &object[key_pos..];
    let colon = tail.find(':')?;
    let tail = tail[colon + 1..].trim_start();
    if tail.starts_with("null") {
        return None;
    }
    let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
    let target: PlayerId = digits.parse().ok()?;
    (target != context.seat && context.alive.contains(&target)).then_some(target)
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
        "白天发言" => Some(Action::Speech {
            text: format!("我是{}，过。", context.role.display_name()),
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

/// 决策时从记忆表取该座位的记忆文本（调用方在锁内取条目后传入）
pub fn autoplay_memory_text(memory: &memory::AgentMemory) -> String {
    memory.render()
}
