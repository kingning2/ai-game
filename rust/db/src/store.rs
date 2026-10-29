//! 房间 / 对局 / 事件流水 CRUD。
//!
//! 设计取舍：本地单用户 SQLite + WAL，单条 SQL 亚毫秒级，调用方直接在
//! async 上下文持 `Mutex<Connection>` 执行即可，无需 spawn_blocking。

use rusqlite::{Connection, OptionalExtension, params, params_from_iter};

use crate::DbError;

/// 房间记录（`rooms` 行 + 席位）
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoomRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub board_id: String,
    /// waiting / playing / finished / dissolved
    pub status: String,
    pub created_at: String,
}

/// 房间席位（发牌后 `role` 为引擎随机分配的真实身份显示名）
#[derive(Debug, Clone, serde::Serialize)]
pub struct SeatRow {
    pub seat: u8,
    pub name: String,
    pub role: Option<String>,
    pub alive: bool,
}

/// 一条事件流水
#[derive(Debug, Clone, serde::Serialize)]
pub struct EventRow {
    pub seq: i64,
    /// speech / wolf_chat / vote / night_action / dawn / died / over / ...
    pub kind: String,
    pub seat: Option<u8>,
    pub player: Option<String>,
    pub content: String,
    pub day: u32,
}

/// 创建房间（幂等房间号由调用方生成；返回 DB id）
pub fn create_room(
    conn: &Connection,
    code: &str,
    name: &str,
    board_id: &str,
    seats: &[(u8, String)],
) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO rooms(code, name, board_id) VALUES (?1, ?2, ?3)",
        params![code, name, board_id],
    )
    .map_err(|e| DbError::wrap(format!("创建房间 {code}"), e))?;
    let room_id = conn.last_insert_rowid();
    let mut stmt = conn
        .prepare("INSERT INTO room_players(room_id, seat, name) VALUES (?1, ?2, ?3)")
        .map_err(|e| DbError::wrap("写入房间席位", e))?;
    for (seat, player) in seats {
        stmt.execute(params![room_id, i64::from(*seat), player])
            .map_err(|e| DbError::wrap(format!("写入席位 {seat}"), e))?;
    }
    Ok(room_id)
}

/// 房间列表（新→旧）
pub fn list_rooms(conn: &Connection, limit: u32) -> Result<Vec<RoomRow>, DbError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, code, name, board_id, status, created_at
             FROM rooms ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| DbError::wrap("查询房间列表", e))?;
    let rows = stmt
        .query_map([i64::from(limit)], |r| {
            Ok(RoomRow {
                id: r.get(0)?,
                code: r.get(1)?,
                name: r.get(2)?,
                board_id: r.get(3)?,
                status: r.get(4)?,
                created_at: r.get(5)?,
            })
        })
        .map_err(|e| DbError::wrap("查询房间列表", e))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| DbError::wrap("读取房间行", e))
}

/// 按房间号取房间（含席位，座位升序）
pub fn get_room(conn: &Connection, code: &str) -> Result<Option<(RoomRow, Vec<SeatRow>)>, DbError> {
    let room = conn
        .query_row(
            "SELECT id, code, name, board_id, status, created_at FROM rooms WHERE code = ?1",
            [code],
            |r| {
                Ok(RoomRow {
                    id: r.get(0)?,
                    code: r.get(1)?,
                    name: r.get(2)?,
                    board_id: r.get(3)?,
                    status: r.get(4)?,
                    created_at: r.get(5)?,
                })
            },
        )
        .optional()
        .map_err(|e| DbError::wrap(format!("查询房间 {code}"), e))?;
    let Some(room) = room else { return Ok(None) };
    let mut stmt = conn
        .prepare(
            "SELECT seat, name, role, alive FROM room_players
             WHERE room_id = ?1 ORDER BY seat",
        )
        .map_err(|e| DbError::wrap("查询房间席位", e))?;
    let seats = stmt
        .query_map([room.id], |r| {
            Ok(SeatRow {
                seat: u8::try_from(r.get::<_, i64>(0)?).unwrap_or(0),
                name: r.get(1)?,
                role: r.get(2)?,
                alive: r.get::<_, i64>(3)? != 0,
            })
        })
        .map_err(|e| DbError::wrap("读取席位行", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| DbError::wrap("读取席位行", e))?;
    Ok(Some((room, seats)))
}

/// 发牌结果落库：回写每个座位的真实身份（引擎随机分配的结果）
pub fn save_deal(conn: &Connection, room_id: i64, seats: &[(u8, String)]) -> Result<(), DbError> {
    let mut stmt = conn
        .prepare("UPDATE room_players SET role = ?3, updated_at = datetime('now','localtime') WHERE room_id = ?1 AND seat = ?2")
        .map_err(|e| DbError::wrap("回写身份", e))?;
    for (seat, role) in seats {
        stmt.execute(params![room_id, i64::from(*seat), role])
            .map_err(|e| DbError::wrap(format!("回写 {seat} 号身份"), e))?;
    }
    conn.execute(
        "UPDATE rooms SET status='playing', updated_at=datetime('now','localtime') WHERE id=?1",
        [room_id],
    )
    .map_err(|e| DbError::wrap("更新房间状态为 playing", e))?;
    Ok(())
}

/// 对局开局落库，返回 game_id
pub fn begin_game(conn: &Connection, room_id: i64, board_id: &str) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO games(room_id, board_id) VALUES (?1, ?2)",
        params![room_id, board_id],
    )
    .map_err(|e| DbError::wrap("写入对局记录", e))?;
    Ok(conn.last_insert_rowid())
}

/// 追加事件流水（增量；调用方按 seq 去重）
pub fn append_events(conn: &Connection, game_id: i64, events: &[EventRow]) -> Result<(), DbError> {
    if events.is_empty() {
        return Ok(());
    }
    let mut stmt = conn
        .prepare(
            "INSERT INTO events(game_id, seq, kind, seat, player, content, day)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(|e| DbError::wrap("写入事件流水", e))?;
    for ev in events {
        stmt.execute(params![
            game_id,
            ev.seq,
            ev.kind,
            ev.seat.map(i64::from),
            ev.player,
            ev.content,
            i64::from(ev.day),
        ])
        .map_err(|e| DbError::wrap(format!("写入事件 seq={}", ev.seq), e))?;
    }
    Ok(())
}

/// 清扫孤儿房：创建后从未开局（waiting）且超过 `keep_minutes` 的房间删除（级联清席位）。
/// 等待态只是建房→开局的毫秒级过渡，不应长期存在；残留即开局失败/双发的垃圾。
pub fn sweep_waiting(conn: &Connection, keep_minutes: i64) -> Result<usize, DbError> {
    let n = conn
        .execute(
            "DELETE FROM rooms WHERE status='waiting'
             AND datetime(created_at, '+' || ?1 || ' minutes') < datetime('now','localtime')",
            params![keep_minutes],
        )
        .map_err(|e| DbError::wrap("清扫等待孤儿房", e))?;
    Ok(n)
}

/// 对局结束：写胜负与结局快照；房间置 finished
pub fn finish_game(
    conn: &Connection,
    game_id: i64,
    room_id: i64,
    winner: &str,
    result_json: &str,
) -> Result<(), DbError> {
    conn.execute(
        "UPDATE games SET winner=?3, result=?4, ended_at=datetime('now','localtime') WHERE id=?1",
        params![game_id, room_id, winner, result_json],
    )
    .map_err(|e| DbError::wrap("写入对局结果", e))?;
    conn.execute(
        "UPDATE rooms SET status='finished', updated_at=datetime('now','localtime') WHERE id=?1",
        [room_id],
    )
    .map_err(|e| DbError::wrap("更新房间状态为 finished", e))?;
    Ok(())
}

/// 已落库的最大事件 seq（增量 flush 基准）；无记录返回 0
pub fn max_event_seq(conn: &Connection, game_id: i64) -> Result<i64, DbError> {
    conn.query_row(
        "SELECT COALESCE(MAX(seq), -1) FROM events WHERE game_id = ?1",
        [game_id],
        |r| r.get::<_, i64>(0),
    )
    .map(|n| n + 1)
    .map_err(|e| DbError::wrap("查询事件水位", e))
}

/// 回放一局（事件按 seq 升序）
pub fn game_events(conn: &Connection, game_id: i64) -> Result<Vec<EventRow>, DbError> {
    let mut stmt = conn
        .prepare(
            "SELECT seq, kind, seat, player, content, day FROM events
             WHERE game_id = ?1 ORDER BY seq",
        )
        .map_err(|e| DbError::wrap("回放事件", e))?;
    let rows = stmt
        .query_map([game_id], |r| {
            Ok(EventRow {
                seq: r.get(0)?,
                kind: r.get(1)?,
                seat: r
                    .get::<_, Option<i64>>(2)?
                    .map(|s| u8::try_from(s).unwrap_or(0)),
                player: r.get(3)?,
                content: r.get(4)?,
                day: u32::try_from(r.get::<_, i64>(5)?).unwrap_or(0),
            })
        })
        .map_err(|e| DbError::wrap("回放事件", e))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| DbError::wrap("读取事件行", e))
}

/// 房间最近一局的定位信息（观战 `?room=` 定位 + 增量落库归属）
#[derive(Debug, Clone)]
pub struct RoomGameRow {
    pub game_id: i64,
    pub room_id: i64,
    pub board_id: String,
    pub winner: Option<String>,
}

/// 房间最近一局（games JOIN rooms 按 code，取该房间 id 最大的一局）
pub fn latest_game_by_room(
    conn: &Connection,
    room_code: &str,
) -> Result<Option<RoomGameRow>, DbError> {
    conn.query_row(
        "SELECT g.id, r.id, r.board_id, g.winner
         FROM games g JOIN rooms r ON r.id = g.room_id
         WHERE r.code = ?1 ORDER BY g.id DESC LIMIT 1",
        [room_code],
        |r| {
            Ok(RoomGameRow {
                game_id: r.get(0)?,
                room_id: r.get(1)?,
                board_id: r.get(2)?,
                winner: r.get(3)?,
            })
        },
    )
    .optional()
    .map_err(|e| DbError::wrap(format!("查询房间 {room_code} 最近一局"), e))
}

/// 房间席位：`(seat, name, role)`，座位升序（降级结算的真实名单）
pub type RosterSeat = (i64, String, Option<String>);

/// 按房间 id 取席位（含发牌后的真实身份，座位升序）
pub fn room_roster_by_id(conn: &Connection, room_id: i64) -> Result<Vec<RosterSeat>, DbError> {
    let mut stmt = conn
        .prepare("SELECT seat, name, role FROM room_players WHERE room_id = ?1 ORDER BY seat")
        .map_err(|e| DbError::wrap(format!("查询房间 {room_id} 席位"), e))?;
    let rows = stmt
        .query_map([room_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|e| DbError::wrap(format!("查询房间 {room_id} 席位"), e))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| DbError::wrap("读取席位行", e))
}

/// 单局结果定位（`?game=` 直查 + 降级结算组装）
#[derive(Debug, Clone)]
pub struct GameResultRow {
    pub game_id: i64,
    pub room_id: i64,
    pub board_id: String,
    pub winner: Option<String>,
    /// 结局快照 JSON（finish_game / finish_stale_games 写入）
    pub result: Option<String>,
}

/// 按对局 id 取结果行（含未完局）
pub fn game_result(conn: &Connection, game_id: i64) -> Result<Option<GameResultRow>, DbError> {
    conn.query_row(
        "SELECT id, room_id, board_id, winner, result FROM games WHERE id = ?1",
        [game_id],
        |r| {
            Ok(GameResultRow {
                game_id: r.get(0)?,
                room_id: r.get(1)?,
                board_id: r.get(2)?,
                winner: r.get(3)?,
                result: r.get(4)?,
            })
        },
    )
    .optional()
    .map_err(|e| DbError::wrap(format!("查询对局 {game_id}"), e))
}

/// 开机清扫：上次运行遗留的未完局（winner IS NULL）标记 stale 结束，房间置 finished。
/// 幂等：已标记过的局不再处理。返回本次标记的局数。
pub fn finish_stale_games(conn: &Connection) -> Result<usize, DbError> {
    const STALE_RESULT: &str = "{\"stale\":true,\"reason\":\"服务器重启，对局中断\"}";
    let swept = conn
        .execute(
            "UPDATE games SET winner='stale', result=?1,
             ended_at=datetime('now','localtime') WHERE winner IS NULL",
            params![STALE_RESULT],
        )
        .map_err(|e| DbError::wrap("清扫未完局", e))?;
    if swept > 0 {
        conn.execute(
            "UPDATE rooms SET status='finished', updated_at=datetime('now','localtime')
             WHERE status='playing'
               AND id IN (SELECT room_id FROM games WHERE winner='stale')",
            [],
        )
        .map_err(|e| DbError::wrap("置房间为 finished", e))?;
    }
    Ok(swept)
}

// ---------------------------------------------------------------------------
// 赛后 MVP 投票
// ---------------------------------------------------------------------------

/// 记录一张 MVP 选票（同局同 voter 重复投票以 UPSERT 覆盖）
pub fn record_mvp_vote(
    conn: &Connection,
    game_id: i64,
    voter: &str,
    candidate: &str,
    reason: &str,
) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO mvp_votes(game_id, voter, candidate, reason) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(game_id, voter)
         DO UPDATE SET candidate = excluded.candidate, reason = excluded.reason",
        params![game_id, voter, candidate, reason],
    )
    .map_err(|e| DbError::wrap(format!("记录 MVP 选票（{voter} → {candidate}）"), e))?;
    Ok(())
}

/// 写入本场 MVP（多数票决出后调用）
pub fn set_game_mvp(conn: &Connection, game_id: i64, mvp: &str) -> Result<(), DbError> {
    conn.execute(
        "UPDATE games SET mvp = ?2 WHERE id = ?1",
        params![game_id, mvp],
    )
    .map_err(|e| DbError::wrap("写入本场 MVP", e))?;
    Ok(())
}

/// 单局 MVP 汇总：`(games.mvp, [(voter, candidate, reason), …])`（选票按落库顺序）。
/// 对局不存在返回 `None`；MVP 未决出时 mvp 为 `None` 但已落库的选票仍返回。
/// 单张选票：`(voter, candidate, reason)`
type MvpVoteRow = (String, String, String);
/// [`game_mvp_summary`] 返回：`(games.mvp, 全部选票)`
type MvpSummary = (Option<String>, Vec<MvpVoteRow>);

pub fn game_mvp_summary(conn: &Connection, game_id: i64) -> Result<Option<MvpSummary>, DbError> {
    let mvp: Option<Option<String>> = conn
        .query_row("SELECT mvp FROM games WHERE id = ?1", [game_id], |r| {
            r.get(0)
        })
        .optional()
        .map_err(|e| DbError::wrap(format!("查询对局 {game_id} 的 MVP"), e))?;
    let Some(mvp) = mvp else {
        return Ok(None); // 对局不存在
    };
    let mut stmt = conn
        .prepare(
            "SELECT voter, candidate, COALESCE(reason, '') FROM mvp_votes
             WHERE game_id = ?1 ORDER BY id",
        )
        .map_err(|e| DbError::wrap("查询 MVP 选票", e))?;
    let votes = stmt
        .query_map([game_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| DbError::wrap("查询 MVP 选票", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| DbError::wrap("读取 MVP 选票行", e))?;
    Ok(Some((mvp, votes)))
}

/// 跨对局胜场排行榜行（按选手聚合所有已结束局）
#[derive(Debug, Clone, serde::Serialize)]
pub struct LeaderboardRow {
    /// 选手名
    pub name: String,
    /// 参与的已结束局数（含 stale 中断局）
    pub games: i64,
    /// 胜场数
    pub wins: i64,
    /// 当选 MVP 次数
    pub mvp_count: i64,
    /// 胜率（0..1）
    pub win_rate: f64,
}

/// 跨对局胜利排行榜：聚合所有已结束局（winner 非空）× 房间席位，仅统计
/// `roster` 白名单内的选手（正式选手名单由 server 传入，测试局的临时玩家
/// 不入榜）。胜
/// 负判定：席位真实身份（发牌后写入 `room_players.role` 显示名）的阵营与
/// `games.winner` 相等即胜。db 不依赖 games 包的 Role 枚举，直接按显示名字符串
/// 映射阵营：狼人 → "werewolf"；其余身份（预言家/女巫/守卫/猎人/白痴/骑士/平民）
/// 全部 → "good"（未发牌的 NULL 身份不算胜）。未分胜负的中断局（winner='stale'）
/// 计入场次但不算胜。排序：胜场 desc → 胜率 desc → 场次 desc。
pub fn leaderboard(conn: &Connection, roster: &[&str]) -> Result<Vec<LeaderboardRow>, DbError> {
    if roster.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = std::iter::repeat_n("?", roster.len())
        .collect::<Vec<_>>()
        .join(", ");
    let mut stmt = conn
        .prepare(&format!(
            "SELECT rp.name,
                    COUNT(*) AS games,
                    SUM(CASE
                        WHEN rp.role = '狼人' AND g.winner = 'werewolf' THEN 1
                        WHEN rp.role IS NOT NULL AND rp.role != '狼人' AND g.winner = 'good' THEN 1
                        ELSE 0
                    END) AS wins,
                    SUM(CASE WHEN g.mvp = rp.name THEN 1 ELSE 0 END) AS mvp_count
             FROM room_players rp
             JOIN games g ON g.room_id = rp.room_id
             WHERE g.winner IS NOT NULL AND rp.name IN ({placeholders})
             GROUP BY rp.name
             ORDER BY wins DESC,
                      CAST(wins AS REAL) / games DESC,
                      games DESC, rp.name",
        ))
        .map_err(|e| DbError::wrap("查询胜利排行榜", e))?;
    let rows = stmt
        .query_map(params_from_iter(roster), |r| {
            let games: i64 = r.get(1)?;
            let wins: i64 = r.get(2)?;
            Ok(LeaderboardRow {
                name: r.get(0)?,
                games,
                wins,
                mvp_count: r.get(3)?,
                win_rate: wins as f64 / games as f64,
            })
        })
        .map_err(|e| DbError::wrap("查询胜利排行榜", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| DbError::wrap("读取排行榜行", e))?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
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
}
