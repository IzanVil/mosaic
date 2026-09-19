//! Apertura y acceso concurrente a la base de datos SQLite.
//!
//! Mosaic es una aplicación de un solo proceso con un único escritor, así que
//! en lugar de un pool se usa un `Mutex<Connection>`. Se reevaluará (r2d2) si
//! las fases 4-5 introducen lecturas concurrentes pesadas.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use directories::ProjectDirs;
use rusqlite::Connection;

use crate::errors::{AppError, Result};

/// Tiempo que SQLite espera a que se libere un bloqueo antes de fallar.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Conexión compartida a la base de datos de la aplicación.
pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    /// Abre (o crea) la base de datos en `path`, creando los directorios padre
    /// si hacen falta, y aplica las migraciones pendientes.
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

    /// Abre una base de datos en memoria ya migrada. Pensado para tests.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::configure(&conn)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.with_conn_mut(crate::db::migrations::apply)?;
        Ok(db)
    }

    /// Aplica los PRAGMA obligatorios a una conexión recién abierta.
    fn configure(conn: &Connection) -> Result<()> {
        conn.busy_timeout(BUSY_TIMEOUT)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        // `journal_mode` devuelve una fila, así que no sirve `pragma_update`.
        // En bases en memoria SQLite ignora WAL y responde "memory": no es un error.
        let mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
        tracing::debug!(journal_mode = %mode, "conexión SQLite configurada");
        Ok(())
    }

    /// Ejecuta `f` con acceso de solo lectura a la conexión.
    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| AppError::Internal("mutex de la base de datos envenenado".into()))?;
        f(&guard)
    }

    /// Ejecuta `f` con acceso exclusivo a la conexión (necesario para transacciones).
    pub fn with_conn_mut<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T>) -> Result<T> {
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| AppError::Internal("mutex de la base de datos envenenado".into()))?;
        f(&mut guard)
    }
}

/// Ruta por defecto de la base de datos, dentro del directorio de datos del SO.
///
/// Linux: `~/.local/share/mosaic/mosaic.db`,
/// macOS: `~/Library/Application Support/dev.izan.mosaic/mosaic.db`,
/// Windows: `%APPDATA%\izan\mosaic\data\mosaic.db`.
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
