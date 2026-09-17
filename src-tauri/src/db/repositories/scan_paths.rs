use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanPath {
    pub id: i64,
    pub path: String,
    pub enabled: bool,
    pub created_at: i64,
    pub last_scan_at: Option<i64>,
}

const COLUMNS: &str = "id, path, enabled, created_at, last_scan_at";

fn row_to_scan_path(row: &Row<'_>) -> rusqlite::Result<ScanPath> {
    Ok(ScanPath {
        id: row.get(0)?,
        path: row.get(1)?,
        enabled: row.get(2)?,
        created_at: row.get(3)?,
        last_scan_at: row.get(4)?,
    })
}

pub fn add(conn: &Connection, path: &str, now: i64) -> Result<ScanPath> {
    conn.execute(
        "INSERT INTO scan_paths (path, enabled, created_at) VALUES (?1, 1, ?2)
         ON CONFLICT(path) DO NOTHING",
        params![path, now],
    )?;

    find_by_path(conn, path)?
        .ok_or_else(|| AppError::Internal(format!("no se pudo registrar la ruta {path}")))
}

pub fn find_by_path(conn: &Connection, path: &str) -> Result<Option<ScanPath>> {
    let sql = format!("SELECT {COLUMNS} FROM scan_paths WHERE path = ?1");
    Ok(conn
        .query_row(&sql, params![path], row_to_scan_path)
        .optional()?)
}

pub fn list(conn: &Connection) -> Result<Vec<ScanPath>> {
    let sql = format!("SELECT {COLUMNS} FROM scan_paths ORDER BY created_at ASC, id ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_scan_path)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_enabled(conn: &Connection) -> Result<Vec<ScanPath>> {
    Ok(list(conn)?.into_iter().filter(|p| p.enabled).collect())
}

pub fn delete(conn: &Connection, id: i64) -> Result<bool> {
    let affected = conn.execute("DELETE FROM scan_paths WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

pub fn set_enabled(conn: &Connection, id: i64, enabled: bool) -> Result<()> {
    let affected = conn.execute(
        "UPDATE scan_paths SET enabled = ?2 WHERE id = ?1",
        params![id, enabled],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("ruta de escaneo {id}")));
    }
    Ok(())
}

pub fn set_last_scan_at(conn: &Connection, id: i64, ts: i64) -> Result<()> {
    let affected = conn.execute(
        "UPDATE scan_paths SET last_scan_at = ?2 WHERE id = ?1",
        params![id, ts],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("ruta de escaneo {id}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn add_registers_a_path() {
        let db = Db::open_in_memory().unwrap();

        let added = db.with_conn(|conn| add(conn, "/code", 1_000)).unwrap();

        assert_eq!(added.path, "/code");
        assert!(added.enabled);
        assert_eq!(added.created_at, 1_000);
        assert_eq!(added.last_scan_at, None);
    }

    #[test]
    fn add_is_idempotent() {
        let db = Db::open_in_memory().unwrap();

        let first = db.with_conn(|conn| add(conn, "/code", 1_000)).unwrap();
        let second = db.with_conn(|conn| add(conn, "/code", 2_000)).unwrap();

        assert_eq!(first, second, "no se duplica ni se pisa created_at");
        assert_eq!(db.with_conn(list).unwrap().len(), 1);
    }

    #[test]
    fn list_returns_paths_in_creation_order() {
        let db = Db::open_in_memory().unwrap();
        db.with_conn(|conn| add(conn, "/code", 1_000)).unwrap();
        db.with_conn(|conn| add(conn, "/trabajo", 2_000)).unwrap();

        let paths = db.with_conn(list).unwrap();

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].path, "/code");
        assert_eq!(paths[1].path, "/trabajo");
    }

    #[test]
    fn list_enabled_skips_disabled_paths() {
        let db = Db::open_in_memory().unwrap();
        let a = db.with_conn(|conn| add(conn, "/code", 1_000)).unwrap();
        db.with_conn(|conn| add(conn, "/trabajo", 2_000)).unwrap();

        db.with_conn(|conn| set_enabled(conn, a.id, false)).unwrap();

        let enabled = db.with_conn(list_enabled).unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].path, "/trabajo");
    }

    #[test]
    fn delete_removes_the_path_and_reports_existence() {
        let db = Db::open_in_memory().unwrap();
        let added = db.with_conn(|conn| add(conn, "/code", 1_000)).unwrap();

        assert!(db.with_conn(|conn| delete(conn, added.id)).unwrap());
        assert!(db.with_conn(list).unwrap().is_empty());
        assert!(!db.with_conn(|conn| delete(conn, added.id)).unwrap());
    }

    #[test]
    fn set_last_scan_at_updates_the_timestamp() {
        let db = Db::open_in_memory().unwrap();
        let added = db.with_conn(|conn| add(conn, "/code", 1_000)).unwrap();

        db.with_conn(|conn| set_last_scan_at(conn, added.id, 5_000))
            .unwrap();

        let stored = db
            .with_conn(|conn| find_by_path(conn, "/code"))
            .unwrap()
            .unwrap();
        assert_eq!(stored.last_scan_at, Some(5_000));
    }

    #[test]
    fn set_last_scan_at_reports_missing_path() {
        let db = Db::open_in_memory().unwrap();
        let err = db
            .with_conn(|conn| set_last_scan_at(conn, 42, 5_000))
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
