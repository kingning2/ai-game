//! SQLite 持久化包：狼人杀房间 / 对局结果 / 全过程事件流水。
//!
//! 单文件库（默认 `%APPDATA%/agent-packages/werewolf.db`），rusqlite bundled
//! 静态编译 SQLite，零外部部署。连接非 `Sync`，调用方用 `Mutex` 包裹并
//! 配合 `tokio::task::spawn_blocking` 使用。
//!
//! 表结构：
//! - `rooms`：房间（房间号、板子、状态 waiting/playing/finished/dissolved）
//! - `room_players`：房间席位（含发牌后的真实身份——身份由引擎随机，此处仅记录）
//! - `games`：对局结果（胜负、结局快照 JSON、本场 MVP）
//! - `mvp_votes`：赛后 MVP 投票（每局每 voter 一票，重复投票 UPSERT 覆盖）
//! - `events`：全过程流水（发言/狼聊/投票/系统事件，可完整回放）

pub use store::{
    EventRow, GameResultRow, LeaderboardRow, RoomGameRow, RoomRow, SeatRow, append_events,
    begin_game, create_room, finish_game, finish_stale_games, game_events, game_mvp_summary,
    game_result, get_room, latest_game_by_room, leaderboard, list_rooms, max_event_seq,
    record_mvp_vote, room_roster_by_id, save_deal, set_game_mvp, sweep_waiting,
};

mod store;

use rusqlite::Connection;

/// 数据库层错误：打开/建表/SQL 执行失败，上下文在文案里、底层原因在 source 链
#[derive(Debug, thiserror::Error)]
#[error("数据库操作失败：{context}")]
pub struct DbError {
    /// 失败环节描述
    context: String,
    #[source]
    source: rusqlite::Error,
}

impl DbError {
    fn wrap(context: impl Into<String>, err: rusqlite::Error) -> Self {
        Self {
            context: context.into(),
            source: err,
        }
    }

    /// 取出底层 rusqlite 错误（调用方需要区分错误类别时用）
    pub fn into_inner(self) -> rusqlite::Error {
        self.source
    }
}

/// 打开（或创建）数据库并确保表结构就绪。
pub fn open(path: &str) -> Result<Connection, DbError> {
    Connection::open(path)
        .map_err(|e| DbError::wrap(format!("打开数据库 {path}"), e))
        .and_then(|conn| init_schema(&conn).map(|_| conn))
}

/// 内存库（测试用）。
pub fn open_in_memory() -> Result<Connection, DbError> {
    Connection::open_in_memory()
        .map_err(|e| DbError::wrap("打开内存库", e))
        .and_then(|conn| init_schema(&conn).map(|_| conn))
}

fn init_schema(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS rooms (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            code        TEXT NOT NULL UNIQUE,
            name        TEXT NOT NULL,
            board_id    TEXT NOT NULL,
            status      TEXT NOT NULL DEFAULT 'waiting',
            created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS room_players (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            room_id    INTEGER NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
            seat       INTEGER NOT NULL,
            name       TEXT NOT NULL,
            role       TEXT,
            alive      INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            UNIQUE (room_id, seat)
        );

        CREATE TABLE IF NOT EXISTS games (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            room_id    INTEGER NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
            board_id   TEXT NOT NULL,
            winner     TEXT,
            result     TEXT,
            mvp        TEXT,
            ended_at   TEXT
        );

        CREATE TABLE IF NOT EXISTS mvp_votes (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id   INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            voter     TEXT NOT NULL,
            candidate TEXT NOT NULL,
            reason    TEXT,
            UNIQUE (game_id, voter)
        );

        CREATE TABLE IF NOT EXISTS events (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id    INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            seq        INTEGER NOT NULL,
            kind       TEXT NOT NULL,
            seat       INTEGER,
            player     TEXT,
            content    TEXT NOT NULL,
            day        INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE INDEX IF NOT EXISTS idx_events_game ON events(game_id, seq);
        CREATE INDEX IF NOT EXISTS idx_rooms_status ON rooms(status, id DESC);
        CREATE INDEX IF NOT EXISTS idx_mvp_votes_game ON mvp_votes(game_id);
        "#,
    )
    .map_err(|e| DbError::wrap("初始化表结构", e))?;

    Ok(())
}

/// 生成房间号：6 位大写数字字母（去掉易混淆的 0/O/1/I）。
pub fn room_code() -> String {
    const ALPHABET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
    let mut code = String::with_capacity(6);
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(1);
    for _ in 0..6 {
        // 简单 LCG 混洗，碰撞概率极低且有 UNIQUE 兜底
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx = ((seed >> 33) as usize) % ALPHABET.len();
        code.push(ALPHABET[idx] as char);
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_memory_creates_tables() {
        let conn = open_in_memory().unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('rooms','room_players','games','events')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 4);
    }

    #[test]
    fn open_file_persists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.db");
        {
            let conn = open(path.to_str().unwrap()).unwrap();
            conn.execute(
                "INSERT INTO rooms(code,name,board_id) VALUES ('ABC234','测试房','standard12')",
                [],
            )
            .unwrap();
        }
        let conn = open(path.to_str().unwrap()).unwrap();
        let name: String = conn
            .query_row("SELECT name FROM rooms WHERE code='ABC234'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(name, "测试房");
    }

    #[test]
    fn room_code_shape() {
        for _ in 0..50 {
            let code = room_code();
            assert_eq!(code.len(), 6);
            assert!(
                code.chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            );
            assert!(
                !code.contains('0')
                    && !code.contains('O')
                    && !code.contains('1')
                    && !code.contains('I')
            );
        }
    }

    #[test]
    fn cascade_delete_players() {
        let conn = open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO rooms(code,name,board_id) VALUES ('XYZ234','房','b')",
            [],
        )
        .unwrap();
        let rid: i64 = conn
            .query_row("SELECT id FROM rooms", [], |r| r.get(0))
            .unwrap();
        conn.execute(
            "INSERT INTO room_players(room_id,seat,name) VALUES (?1,1,'甲'),(?1,2,'乙')",
            [&rid],
        )
        .unwrap();
        conn.execute("DELETE FROM rooms WHERE id=?1", [&rid])
            .unwrap();
        let n: i64 = conn
            .query_row("SELECT count(*) FROM room_players", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0);
    }
}
