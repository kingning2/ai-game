//! 赛后 MVP 投票：胜负落库后让全部参赛选手（不限存活）各投一票，多数票决出本场 MVP。
//!
//! 流程：`spawn`（胜负首次落库时触发）→ 独立 tokio 任务（不阻塞 autoplay 收尾与
//! status 轮询）→ 从 db 取席位/身份/事件回顾 → 每人用各自的选手模型配置投一票
//! （JSON `{"vote":"座位号或名字","reason":"…"}`，解析失败重试 1 次，再失败弃权）
//! → 多数票决出 MVP → `set_game_mvp` + 逐票 `record_mvp_vote` 落库。
//!
//! 失败兜底：投票整体失败（如选手均未配置 LLM、db 不可用）只记 warn，MVP 保持
//! 无值（status 侧维持 pending），绝不影响其他对局。
//!
//! 并发选择：12 票按 4 一批发（[`VOTE_CONCURRENCY`]）——全顺序会拖长终局后的
//! pending 等待（每票数秒起），12 票齐发又容易打满选手 provider 的并发配额，
//! 分批错峰是等待时长与请求压力的折中。

use std::time::Duration;

use agent::event::AgentEvent;
use agent::{ChatMessage, ChatRequest, build_provider};
use state::{AppState, MvpPhase};

use crate::orchestr::autoplay::first_json_object;

/// 单票 LLM 调用超时（决策级；与 autoplay 决策同一量级）
const VOTE_TIMEOUT: Duration = Duration::from_secs(60);
/// 每批并发投票数（见模块注释的并发选择）
const VOTE_CONCURRENCY: usize = 4;

/// 参赛选手快照（db 席位行；座位 1 起）
struct Voter {
    seat: u8,
    name: String,
    /// 真实身份显示名（结束后公开）
    role: String,
}

/// 胜负落库后启动本局的 MVP 投票任务（同步入口，内部 tokio::spawn 不阻塞调用方）
pub fn spawn(state: &AppState, game_id: i64, room_id: i64) {
    let state = state.clone();
    tokio::spawn(async move {
        run(&state, game_id, room_id).await;
    });
}

/// 投票主流程：快照 → 分批投票 → 多数票决出 → 落库
async fn run(state: &AppState, game_id: i64, room_id: i64) {
    // 状态先行：status 轮询立刻能看到「投票中」
    state.game.set_mvp_phase(game_id, MvpPhase::Pending);

    // 锁内只做同步快照收集（Mutex 闭包内不得 .await）
    let snapshot = {
        let guard = state
            .db
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(conn) = guard.as_ref() else {
            tracing::warn!(game_id, "db 不可用，本局 MVP 投票取消");
            return;
        };
        let voters = match db::room_roster_by_id(conn, room_id) {
            Ok(rows) => rows
                .into_iter()
                .map(|(seat, name, role)| Voter {
                    seat: u8::try_from(seat).unwrap_or(0),
                    name,
                    role: role.unwrap_or_else(|| "未知".to_string()),
                })
                .collect::<Vec<_>>(),
            Err(err) => {
                tracing::warn!(game_id, "读取席位失败，本局 MVP 投票取消：{}", err);
                return;
            }
        };
        let events = db::game_events(conn, game_id).unwrap_or_default();
        (voters, events)
    };
    let (voters, events) = snapshot;
    if voters.is_empty() {
        tracing::warn!(game_id, "无席位记录，本局 MVP 投票取消");
        return;
    }
    // 对局事件回顾（落库时的渲染文本，上帝视角；截断防爆 prompt）
    let summary = events
        .iter()
        .map(|ev| {
            let seat = ev.seat.map(|s| format!("{s}号 ")).unwrap_or_default();
            format!("[{}] {}{}", ev.kind, seat, ev.content)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let summary = truncate(&summary, 6000);

    // 分批并发投票；join_next 返回完成序，与先后无关（平票按座位号裁决）
    let mut votes: Vec<(String, String)> = Vec::new(); // (voter, candidate)
    for chunk in voters.chunks(VOTE_CONCURRENCY) {
        let mut set = tokio::task::JoinSet::new();
        for voter in chunk {
            let state = state.clone();
            let voter = Voter {
                seat: voter.seat,
                name: voter.name.clone(),
                role: voter.role.clone(),
            };
            let candidates: Vec<(u8, String, String)> = voters
                .iter()
                .map(|v| (v.seat, v.name.clone(), v.role.clone()))
                .collect();
            let summary = summary.clone();
            set.spawn(async move { collect_vote(&state, &voter, &candidates, &summary).await });
        }
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok(Some((voter, candidate, reason))) => {
                    if let Err(err) = record_vote(state, game_id, &voter, &candidate, &reason) {
                        tracing::warn!(game_id, "MVP 选票落库失败：{}", err);
                        continue;
                    }
                    votes.push((voter, candidate));
                }
                Ok(None) => {}
                Err(err) => tracing::warn!(game_id, "MVP 投票任务 panic：{err}"),
            }
        }
    }
    if votes.is_empty() {
        tracing::warn!(
            game_id,
            "无人成功投票（选手模型未配置或全部弃权），MVP 保持无值"
        );
        return; // phase 维持 Pending：前端按 90s 封顶展示「统计超时」
    }

    // 多数票决出 MVP；平票取座位号最小者（确定性裁决，写明不按先到）
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for (_, candidate) in &votes {
        *counts.entry(candidate.as_str()).or_insert(0) += 1;
    }
    let best = counts.values().copied().max().unwrap_or(0);
    let seat_of = |name: &str| {
        voters
            .iter()
            .find(|v| v.name == name)
            .map(|v| v.seat)
            .unwrap_or(u8::MAX)
    };
    let mvp = counts
        .iter()
        .filter(|(_, n)| **n == best)
        .map(|(name, _)| name)
        .min_by_key(|name| seat_of(name))
        .cloned();
    let Some(mvp) = mvp else {
        tracing::warn!(game_id, "选票统计异常，MVP 保持无值");
        return;
    };
    if let Err(err) = set_mvp(state, game_id, mvp) {
        tracing::warn!(game_id, "MVP 落库失败：{}", err);
        return;
    }
    state.game.set_mvp_phase(game_id, MvpPhase::Done);
    tracing::info!(game_id, mvp, votes = votes.len(), "本场 MVP 已决出");
}

/// 单票：构建 prompt → LLM（锁外）→ 解析（失败重试 1 次）→ 候选校验。
/// 返回 `(voter, candidate, reason)`；弃权（LLM 不可用 / 解析失败 / 非法目标）返回 `None`。
async fn collect_vote(
    state: &AppState,
    voter: &Voter,
    candidates: &[(u8, String, String)],
    summary: &str,
) -> Option<(String, String, String)> {
    let roster: Vec<(u8, String)> = candidates
        .iter()
        .map(|(seat, name, _)| (*seat, name.clone()))
        .collect();
    let Some(mut reply) = chat_text(
        state,
        vote_messages(voter, candidates, summary),
        &voter.name,
    )
    .await
    else {
        tracing::warn!(
            seat = voter.seat,
            name = voter.name,
            "MVP 投票 LLM 调用失败，弃权"
        );
        return None;
    };
    // 共两次尝试：首答解析失败后重试 1 次，再失败弃权（重试调用失败也直接弃权）
    for attempt in 0..2 {
        if let Some(vote) = parse_vote(&reply, voter, &roster) {
            return Some((voter.name.clone(), vote.0, vote.1));
        }
        tracing::debug!(
            seat = voter.seat,
            attempt,
            reply = %truncate(&reply, 300),
            "MVP 投票输出未解析"
        );
        let Some(next) = chat_retry(state, voter, candidates, summary).await else {
            tracing::warn!(
                seat = voter.seat,
                name = voter.name,
                "MVP 投票重试调用失败，弃权"
            );
            return None;
        };
        reply = next;
    }
    tracing::warn!(
        seat = voter.seat,
        name = voter.name,
        reply = %truncate(&reply, 300),
        "MVP 投票解析失败，弃权"
    );
    None
}

/// 单轮流式调用，拼接文本增量；返回原始回复（带 60s 决策级超时）
async fn chat_text(state: &AppState, messages: Vec<ChatMessage>, name: &str) -> Option<String> {
    let global = state.agent.config();
    let config = state.roster.resolve(name, &global);
    let provider = build_provider(&config).ok()?;
    let request = ChatRequest {
        model: None,
        messages,
        thinking: config.thinking,
    };
    let mut stream = provider.chat_stream(request).await.ok()?;
    let reply = tokio::time::timeout(VOTE_TIMEOUT, async {
        let mut reply = String::new();
        while let Ok(Some(event)) = stream.next_event().await {
            if let AgentEvent::TextDelta { delta } = event {
                reply.push_str(&delta);
            }
        }
        reply
    })
    .await
    .ok()?;
    Some(reply)
}

fn vote_messages(
    voter: &Voter,
    candidates: &[(u8, String, String)],
    summary: &str,
) -> Vec<ChatMessage> {
    let roster_text = candidates
        .iter()
        .map(|(seat, name, role)| format!("{seat}号 {name}（{role}）"))
        .collect::<Vec<_>>()
        .join("；");
    let candidates_text = candidates
        .iter()
        .map(|(seat, name, _)| format!("{seat}号 {name}"))
        .collect::<Vec<_>>()
        .join("；");
    vec![
        ChatMessage::system(format!(
            "你是 AI 狼人杀选手「{}」，坐在 {} 号位，本局身份是{}。对局已经结束，现在进行赛后 MVP 投票。只输出一个 JSON 对象，不要 markdown 代码块，不要解释。",
            voter.name, voter.seat, voter.role
        )),
        ChatMessage::user(format!(
            "【全场真实身份（对局结束已公开）】{roster_text}\n【本场表现最佳候选（可投任何人，不能投自己）】{candidates_text}\n【对局回顾】\n{summary}\n\n【要求】选出一名本场表现最佳的玩家为 MVP，不能投自己。只输出 JSON：{{\"vote\":\"座位号或名字\",\"reason\":\"一句话理由\"}}"
        )),
    ]
}

/// 解析失败后的重试轮：追加纠错消息
async fn chat_retry(
    state: &AppState,
    voter: &Voter,
    candidates: &[(u8, String, String)],
    summary: &str,
) -> Option<String> {
    let mut messages = vote_messages(voter, candidates, summary);
    messages.push(ChatMessage::user(
        "你的上一个输出无法解析。请重新只输出一个合法 JSON：{\"vote\":\"座位号或名字\",\"reason\":\"一句话理由\"}。",
    ));
    chat_text(state, messages, &voter.name).await
}

/// 解析回复：取首个 JSON 对象的 `vote`（座位号数字或名字）与 `reason`；
/// 校验目标在名册内且不是自己
fn parse_vote(reply: &str, voter: &Voter, roster: &[(u8, String)]) -> Option<(String, String)> {
    let object = first_json_object(reply)?;
    let raw = vote_field(&object)?.trim().to_string();
    let reason = object
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    // 座位号优先（容忍「3号」这类后缀：取前导数字）；否则按名字精确匹配
    let digits: String = raw.chars().take_while(char::is_ascii_digit).collect();
    let candidate = if digits.is_empty() {
        roster
            .iter()
            .find(|(_, name)| *name == raw)
            .map(|(_, name)| name.clone())
    } else {
        let seat = digits.parse::<u8>().ok()?;
        roster
            .iter()
            .find(|(s, _)| *s == seat)
            .map(|(_, name)| name.clone())
    }?;
    (candidate != voter.name).then_some((candidate, reason))
}

/// `vote` 字段取值：数字（含字符串包裹的数字）或名字
fn vote_field(object: &serde_json::Map<String, serde_json::Value>) -> Option<String> {
    match object.get("vote")? {
        serde_json::Value::Number(num) => Some(num.to_string()),
        serde_json::Value::String(text) => Some(text.clone()),
        _ => None,
    }
}

/// 按 UTF-8 字符数截断（对局回顾防爆 prompt）
fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let cut: String = text.chars().take(max_chars).collect();
    format!("{cut}\n…（后续事件略）")
}

/// db Mutex 锁内只做同步 IO；`Err` 为可读错误文本
fn db_write<T>(
    state: &AppState,
    write: impl FnOnce(&rusqlite::Connection) -> Result<T, db::DbError>,
) -> Result<T, String> {
    let guard = state
        .db
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(conn) = guard.as_ref() else {
        return Err("db 不可用".to_string());
    };
    write(conn).map_err(|e| e.to_string())
}

/// 选票落库
fn record_vote(
    state: &AppState,
    game_id: i64,
    voter: &str,
    candidate: &str,
    reason: &str,
) -> Result<(), String> {
    db_write(state, |conn| {
        db::record_mvp_vote(conn, game_id, voter, candidate, reason)
    })
}

/// MVP 落库
fn set_mvp(state: &AppState, game_id: i64, mvp: &str) -> Result<(), String> {
    db_write(state, |conn| db::set_game_mvp(conn, game_id, mvp))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voter(name: &str, seat: u8) -> Voter {
        Voter {
            seat,
            name: name.to_string(),
            role: "villager".to_string(),
        }
    }

    #[test]
    fn parse_vote_accepts_seat_number_and_name() {
        let roster = vec![
            (1, "甲".to_string()),
            (2, "乙".to_string()),
            (3, "丙".to_string()),
        ];
        let me = voter("甲", 1);
        // 座位号（JSON 字符串包裹）
        let (cand, reason) =
            parse_vote(r#"{"vote":"3","reason":"带领好人翻盘"}"#, &me, &roster).unwrap();
        assert_eq!(cand, "丙");
        assert_eq!(reason, "带领好人翻盘");
        // 名字
        let (cand, _) = parse_vote(r#"{"vote":"乙","reason":"发言清晰"}"#, &me, &roster).unwrap();
        assert_eq!(cand, "乙");
        // 座位号带后缀（「3号」）
        let (cand, _) = parse_vote(r#"{"vote":"3号","reason":"狼刀稳准"}"#, &me, &roster).unwrap();
        assert_eq!(cand, "丙");
    }

    #[test]
    fn parse_vote_rejects_self_and_unknown_and_noise() {
        let roster = vec![(1, "甲".to_string()), (2, "乙".to_string())];
        let me = voter("甲", 1);
        // 不能投自己
        assert!(parse_vote(r#"{"vote":"甲","reason":""}"#, &me, &roster).is_none());
        assert!(parse_vote(r#"{"vote":"1","reason":""}"#, &me, &roster).is_none());
        // 名册外
        assert!(parse_vote(r#"{"vote":"9","reason":""}"#, &me, &roster).is_none());
        // 无 JSON / 无 vote 字段
        assert!(parse_vote("我觉得丙不错", &me, &roster).is_none());
        assert!(parse_vote(r#"{"reason":"只有理由"}"#, &me, &roster).is_none());
    }

    #[test]
    fn parse_vote_extracts_json_from_markdown_fence() {
        let roster = vec![(1, "甲".to_string()), (2, "乙".to_string())];
        let me = voter("甲", 1);
        let reply = "好的，我的投票如下：\n```json\n{\"vote\":\"2\",\"reason\":\"逻辑最强\"}\n```";
        let (cand, reason) = parse_vote(reply, &me, &roster).unwrap();
        assert_eq!(cand, "乙");
        assert_eq!(reason, "逻辑最强");
    }

    #[test]
    fn truncate_limits_chars_without_panic() {
        assert_eq!(truncate("abcdef", 3), "abc\n…（后续事件略）");
        assert_eq!(truncate("短的", 10), "短的");
        // 多字节字符按 char 截断
        let long = "狼".repeat(50);
        assert_eq!(truncate(&long, 10).chars().count(), 10 + 9);
    }
}
