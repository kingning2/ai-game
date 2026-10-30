//! lib.rs 的单元测试（由内联 mod tests 迁入）。

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
