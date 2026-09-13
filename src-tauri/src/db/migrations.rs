//! Migraciones de esquema, embebidas en el binario.
//!
//! Para añadir una migración: crear `migrations/NNN_nombre.sql` e incluirla en
//! [`MIGRATIONS`]. El orden del vector es el orden de aplicación y no debe
//! reordenarse nunca, solo crecer por el final.

use std::sync::LazyLock;

use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::errors::Result;

static MIGRATIONS: LazyLock<Migrations<'static>> = LazyLock::new(|| {
    Migrations::new(vec![M::up(include_str!(
        "../../migrations/001_initial.sql"
    ))])
});

/// Lleva el esquema de `conn` a la última versión conocida.
pub fn apply(conn: &mut Connection) -> Result<()> {
    let before = MIGRATIONS.current_version(conn)?;
    MIGRATIONS.to_latest(conn)?;
    let after = MIGRATIONS.current_version(conn)?;

    if before == after {
        tracing::debug!(version = ?after, "esquema ya actualizado");
    } else {
        tracing::info!(from = ?before, to = ?after, "migraciones aplicadas");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_valid() {
        MIGRATIONS.validate().unwrap();
    }

    #[test]
    fn apply_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply(&mut conn).unwrap();
        apply(&mut conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table'
                 AND name IN ('scan_paths', 'projects', 'tags', 'project_tags',
                              'git_status_cache', 'settings')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            count, 6,
            "deben existir las seis tablas del esquema inicial"
        );
    }

    #[test]
    fn projects_table_has_missing_and_last_seen_columns() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply(&mut conn).unwrap();

        let mut stmt = conn.prepare("PRAGMA table_info(projects)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .map(std::result::Result::unwrap)
            .collect();

        assert!(columns.contains(&"missing".to_string()));
        assert!(columns.contains(&"last_seen_at".to_string()));
    }
}
