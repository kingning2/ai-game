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
#[path = "store_tests.rs"]
mod tests;
