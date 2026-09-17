use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use directories::ProjectDirs;
use rusqlite::Connection;

use crate::errors::{AppError, Result};

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Self::configure(&conn)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.with_conn_mut(crate::db::migrations::apply)?;
        tracing::info!(path = %path.display(), "base de datos lista");
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::configure(&conn)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.with_conn_mut(crate::db::migrations::apply)?;
        Ok(db)
    }

    fn configure(conn: &Connection) -> Result<()> {
        conn.busy_timeout(BUSY_TIMEOUT)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
        tracing::debug!(journal_mode = %mode, "conexión SQLite configurada");
        Ok(())
    }

    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| AppError::Internal("mutex de la base de datos envenenado".into()))?;
        f(&guard)
    }

    pub fn with_conn_mut<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T>) -> Result<T> {
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| AppError::Internal("mutex de la base de datos envenenado".into()))?;
        f(&mut guard)
    }
}

pub fn default_db_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "izan", "mosaic").ok_or_else(|| {
        AppError::Internal("no se pudo determinar el directorio de datos del sistema".into())
    })?;
    Ok(dirs.data_dir().join("mosaic.db"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_db_applies_migrations_and_enforces_foreign_keys() {
        let db = Db::open_in_memory().unwrap();

        let tables: i64 = db
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'projects'",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(tables, 1);

        let fk: i64 = db
            .with_conn(|conn| Ok(conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?))
            .unwrap();
        assert_eq!(fk, 1, "las foreign keys deben estar activadas");
    }

    #[test]
    fn open_creates_parent_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("anidado/mosaic.db");

        let db = Db::open(&path).unwrap();
        assert!(path.exists());

        drop(db);
    }
}
