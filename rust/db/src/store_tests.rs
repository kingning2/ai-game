//! store.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use crate::open_in_memory;

/// 测试用正式选手名单（对应线上 state::ROSTER_PLAYERS 的角色）
const ROSTER: [&str; 3] = ["豆包", "GPT-6.1", "Kimi"];

fn setup() -> Connection {
    let conn = open_in_memory().unwrap();
    create_room(
        &conn,
        "ABC234",
        "测试房",
        "standard12",
        &[(1, "豆包".into()), (2, "GPT-6.1".into())],
    )
    .unwrap();
    conn
}

#[test]
fn room_lifecycle() {
    let conn = setup();
    let list = list_rooms(&conn, 10).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].status, "waiting");

    let (room, seats) = get_room(&conn, "ABC234").unwrap().unwrap();
    assert_eq!(seats.len(), 2);
    assert_eq!(seats[0].name, "豆包");
    assert!(seats[0].role.is_none());
    assert_eq!(room.id, list[0].id);
}

#[test]
fn deal_and_game_flow() {
    let conn = setup();
    let (room, seats) = get_room(&conn, "ABC234").unwrap().unwrap();
    assert_eq!(seats.len(), 2);
    save_deal(&conn, room.id, &[(1, "狼人".into()), (2, "预言家".into())]).unwrap();
    let (_, seats) = get_room(&conn, "ABC234").unwrap().unwrap();
    assert_eq!(seats[0].role.as_deref(), Some("狼人"));

    let gid = begin_game(&conn, room.id, "standard12").unwrap();
    append_events(
        &conn,
        gid,
        &[
            EventRow {
                seq: 0,
                kind: "started".into(),
                seat: None,
                player: None,
                content: "开局".into(),
                day: 1,
            },
            EventRow {
                seq: 1,
                kind: "speech".into(),
                seat: Some(1),
                player: Some("豆包".into()),
                content: "我是好人".into(),
                day: 1,
            },
        ],
    )
    .unwrap();
    // 增量：水位 2，重复 flush 0 条
    assert_eq!(max_event_seq(&conn, gid).unwrap(), 2);
    append_events(&conn, gid, &[]).unwrap();
    let events = game_events(&conn, gid).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].player.as_deref(), Some("豆包"));

    finish_game(&conn, gid, room.id, "werewolf", "{\"day\":3}").unwrap();
    let list = list_rooms(&conn, 10).unwrap();
    assert_eq!(list[0].status, "finished");
}

#[test]
fn get_room_missing() {
    let conn = setup();
    assert!(get_room(&conn, "ZZZZ99").unwrap().is_none());
}

#[test]
fn sweep_waiting_removes_only_stale() {
    let conn = setup();
    create_room(
        &conn,
        "AAA111",
        "老孤儿",
        "newbie_six",
        &[(1u8, "豆包".to_string())],
    )
    .unwrap();
    // 把创建时间拨旧到超时限
    conn.execute(
        "UPDATE rooms SET created_at=datetime('now','-30 minutes') WHERE code='AAA111'",
        [],
    )
    .unwrap();
    create_room(
        &conn,
        "BBB222",
        "新房间",
        "newbie_six",
        &[(1u8, "豆包".to_string())],
    )
    .unwrap();
    assert_eq!(sweep_waiting(&conn, 10).unwrap(), 1);
    let left: Vec<String> = list_rooms(&conn, 10)
        .unwrap()
        .into_iter()
        .map(|r| r.code)
        .collect();
    // 超时孤儿房被删；新房间与 setup 种子房保留
    assert!(!left.contains(&"AAA111".to_string()));
    assert_eq!(left.len(), 2);
}

#[test]
fn latest_game_and_roster_by_room() {
    let conn = setup();
    let (room, _) = get_room(&conn, "ABC234").unwrap().unwrap();
    save_deal(&conn, room.id, &[(1, "狼人".into()), (2, "预言家".into())]).unwrap();
    let first = begin_game(&conn, room.id, "standard12").unwrap();
    let second = begin_game(&conn, room.id, "standard12").unwrap();
    finish_game(&conn, second, room.id, "good", "{\"day\":2}").unwrap();

    // 最近一局 = id 最大者
    let found = latest_game_by_room(&conn, "ABC234").unwrap().unwrap();
    assert_eq!(found.game_id, second);
    assert_eq!(found.room_id, room.id);
    assert_eq!(found.winner.as_deref(), Some("good"));
    // 第一局未结束，也不影响「取最新」
    assert_ne!(found.game_id, first);

    let roster = room_roster_by_id(&conn, room.id).unwrap();
    assert_eq!(roster.len(), 2);
    assert_eq!(roster[0], (1, "豆包".into(), Some("狼人".into())));
    assert_eq!(roster[1].0, 2);
    let by_id = room_roster_by_id(&conn, room.id).unwrap();
    assert_eq!(by_id.len(), 2);

    // 无记录房间 → None / 空
    assert!(latest_game_by_room(&conn, "ZZZZ99").unwrap().is_none());
}

#[test]
fn game_result_and_stale_sweep() {
    let conn = setup();
    let (room, _) = get_room(&conn, "ABC234").unwrap().unwrap();
    let gid = begin_game(&conn, room.id, "standard12").unwrap();
    conn.execute("UPDATE rooms SET status='playing' WHERE id=?1", [room.id])
        .unwrap();

    // 未完局 → 清扫后 stale
    assert_eq!(finish_stale_games(&conn).unwrap(), 1);
    let row = game_result(&conn, gid).unwrap().unwrap();
    assert_eq!(row.winner.as_deref(), Some("stale"));
    assert!(row.result.as_deref().unwrap_or("").contains("stale"));
    let status: String = conn
        .query_row("SELECT status FROM rooms WHERE id=?1", [room.id], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(status, "finished");

    // 幂等：已标记的不重复处理
    assert_eq!(finish_stale_games(&conn).unwrap(), 0);

    // 已完局不受清扫影响
    let (room2, _) = { get_room(&conn, "ABC234").unwrap().unwrap() };
    let done = begin_game(&conn, room2.id, "standard12").unwrap();
    finish_game(&conn, done, room2.id, "werewolf", "{\"day\":3}").unwrap();
    assert_eq!(finish_stale_games(&conn).unwrap(), 0);
    let row = game_result(&conn, done).unwrap().unwrap();
    assert_eq!(row.winner.as_deref(), Some("werewolf"));
}

/// 建房 + 发牌 + 开局 + 结算的快捷序列（返回 room_id）
fn play_and_finish(
    conn: &Connection,
    code: &str,
    seats: &[(u8, &str)],
    roles: &[(u8, &str)],
    winner: &str,
) -> i64 {
    create_room(
        conn,
        code,
        "测试房",
        "standard12",
        &seats
            .iter()
            .map(|(s, n)| (*s, (*n).to_string()))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let (room, _) = get_room(conn, code).unwrap().unwrap();
    save_deal(
        conn,
        room.id,
        &roles
            .iter()
            .map(|(s, r)| (*s, (*r).to_string()))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let gid = begin_game(conn, room.id, "standard12").unwrap();
    finish_game(conn, gid, room.id, winner, "{}").unwrap();
    room.id
}

#[test]
fn leaderboard_aggregates_across_rooms() {
    let conn = setup();
    // 房 1：狼人胜——豆包（狼）胜，GPT-6.1（平民）败
    play_and_finish(
        &conn,
        "ROOM01",
        &[(1, "豆包"), (2, "GPT-6.1")],
        &[(1, "狼人"), (2, "平民")],
        "werewolf",
    );
    // 房 2：好人胜——豆包（预言家）胜；GPT-6.1（狼）败；Kimi 未参赛房
    play_and_finish(
        &conn,
        "ROOM02",
        &[(1, "豆包"), (2, "GPT-6.1"), (3, "Kimi")],
        &[(1, "预言家"), (2, "狼人"), (3, "女巫")],
        "good",
    );
    // 房 3：中断局（stale）——计入场次，无人胜；MVP 缺席
    play_and_finish(
        &conn,
        "ROOM03",
        &[(1, "豆包"), (2, "Kimi")],
        &[(1, "平民"), (2, "猎人")],
        "stale",
    );

    let board = leaderboard(&conn, &ROSTER).unwrap();
    let by_name = |name: &str| board.iter().find(|r| r.name == name).unwrap();
    // 豆包：3 局 2 胜（胜率 2/3）
    let doubao = by_name("豆包");
    assert_eq!((doubao.games, doubao.wins, doubao.mvp_count), (3, 2, 0));
    assert!((doubao.win_rate - 2.0 / 3.0).abs() < 1e-9);
    // GPT-6.1：2 局 0 胜（两局都是狼且都败）；Kimi：2 局 1 胜（女巫随好人胜，stale 局不算胜）
    assert_eq!((by_name("GPT-6.1").games, by_name("GPT-6.1").wins), (2, 0));
    assert_eq!((by_name("Kimi").games, by_name("Kimi").wins), (2, 1));
    // 未发牌（role NULL）不算胜
    create_room(&conn, "ROOM04", "房", "standard12", &[(1, "豆包".into())]).unwrap();
    let (room, _) = get_room(&conn, "ROOM04").unwrap().unwrap();
    let gid = begin_game(&conn, room.id, "standard12").unwrap();
    finish_game(&conn, gid, room.id, "good", "{}").unwrap();
    let doubao = leaderboard(&conn, &ROSTER)
        .unwrap()
        .into_iter()
        .find(|r| r.name == "豆包")
        .unwrap();
    assert_eq!((doubao.games, doubao.wins), (4, 2));
    assert_eq!(board.len(), 3);
    // 排序：豆包（2 胜）第一
    let board = leaderboard(&conn, &ROSTER).unwrap();
    assert_eq!(board[0].name, "豆包");
}

#[test]
fn leaderboard_filters_out_non_roster_players() {
    // 白名单外（测试局临时玩家）不入榜：正式名单传什么，榜里就只有谁
    let conn = setup();
    play_and_finish(
        &conn,
        "ROOM01",
        &[(1, "豆包"), (2, "Alpha"), (3, "P1")],
        &[(1, "狼人"), (2, "平民"), (3, "预言家")],
        "werewolf",
    );
    let board = leaderboard(&conn, &["豆包"]).unwrap();
    assert_eq!(board.len(), 1);
    assert_eq!(board[0].name, "豆包");
    // 空名单 → 空榜（不报错）
    assert!(leaderboard(&conn, &[]).unwrap().is_empty());
}

#[test]
fn mvp_votes_upsert_and_summary() {
    let conn = setup();
    let room = play_and_finish(
        &conn,
        "MVP001",
        &[(1, "豆包"), (2, "GPT-6.1"), (3, "Kimi")],
        &[(1, "狼人"), (2, "预言家"), (3, "女巫")],
        "good",
    );
    let games: Vec<i64> = conn
        .prepare("SELECT id FROM games WHERE room_id = ?1")
        .unwrap()
        .query_map([room], |r| r.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let gid = games[0];

    // 未投票时：mvp 为 None 但返回空选票列表
    let (mvp, votes) = game_mvp_summary(&conn, gid).unwrap().unwrap();
    assert_eq!(mvp, None);
    assert!(votes.is_empty());

    // 投票 + 重复投票覆盖
    record_mvp_vote(&conn, gid, "豆包", "GPT-6.1", "发言缈辑清晰").unwrap();
    record_mvp_vote(&conn, gid, "豆包", "Kimi", "改投带飞全场的女巫").unwrap();
    record_mvp_vote(&conn, gid, "GPT-6.1", "GPT-6.1", "不好意思投自己").unwrap();
    let (mvp, votes) = game_mvp_summary(&conn, gid).unwrap().unwrap();
    assert_eq!(mvp, None);
    assert_eq!(votes.len(), 2, "同 voter 覆盖不重复计");
    assert_eq!(
        votes[0],
        ("豆包".into(), "Kimi".into(), "改投带飞全场的女巫".into())
    );

    // 决出 MVP → 汇总与 mvp_count
    set_game_mvp(&conn, gid, "Kimi").unwrap();
    let (mvp, _) = game_mvp_summary(&conn, gid).unwrap().unwrap();
    assert_eq!(mvp.as_deref(), Some("Kimi"));
    // 不存在的对局 → None
    assert!(game_mvp_summary(&conn, 999).unwrap().is_none());
    // 排行榜 MVP 计数
    let row = leaderboard(&conn, &ROSTER)
        .unwrap()
        .into_iter()
        .find(|r| r.name == "Kimi")
        .unwrap();
    assert_eq!(row.mvp_count, 1);
    assert_eq!(row.wins, 1, "Kimi（女巫）在好人胜局中获胜");
}
