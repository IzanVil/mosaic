//! Repositorio de proyectos.
//!
//! # Semántica de marcas de tiempo
//!
//! - `created_at`: se fija en la inserción y no se modifica jamás.
//! - `updated_at`: solo avanza si cambian metadatos propios del proyecto
//!   (nombre, lenguaje primario, condición de repositorio Git o notas).
//! - `last_seen_at`: avanza cada vez que el escáner ve el proyecto en disco.
//! - `missing`: `0` cuando el escáner lo ve, `1` cuando deja de verlo.
//!
//! Ver `missing` nunca borra datos: las etiquetas y notas del usuario
//! sobreviven a que el proyecto desaparezca del disco.

use std::collections::HashSet;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::core::project::{DiscoveredProject, Project};
use crate::errors::{AppError, Result};

/// Resultado de un upsert, para que el escáner pueda construir su resumen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpsertOutcome {
    pub id: i64,
    /// El proyecto no existía y se ha insertado.
    pub created: bool,
    /// Cambió algún metadato propio, de modo que `updated_at` ha avanzado.
    pub metadata_changed: bool,
}

/// Columnas de `projects` en el orden que espera [`row_to_project`].
const COLUMNS: &str = "id, name, path, is_git_repo, primary_language, last_opened_at,
     pinned, notes, missing, last_seen_at, created_at, updated_at";

fn row_to_project(row: &Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        is_git_repo: row.get(3)?,
        primary_language: row.get(4)?,
        last_opened_at: row.get(5)?,
        pinned: row.get(6)?,
        notes: row.get(7)?,
        missing: row.get(8)?,
        last_seen_at: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

/// Busca un proyecto por su ruta absoluta.
pub fn find_by_path(conn: &Connection, path: &str) -> Result<Option<Project>> {
    let sql = format!("SELECT {COLUMNS} FROM projects WHERE path = ?1");
    Ok(conn
        .query_row(&sql, params![path], row_to_project)
        .optional()?)
}

/// Recupera un proyecto por su identificador.
pub fn get_by_id(conn: &Connection, id: i64) -> Result<Project> {
    let sql = format!("SELECT {COLUMNS} FROM projects WHERE id = ?1");
    conn.query_row(&sql, params![id], row_to_project)
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("proyecto {id}")))
}

/// Devuelve todos los proyectos: primero los fijados, luego por nombre.
pub fn list_all(conn: &Connection) -> Result<Vec<Project>> {
    let sql =
        format!("SELECT {COLUMNS} FROM projects ORDER BY pinned DESC, name COLLATE NOCASE ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_project)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Número total de proyectos registrados.
pub fn count(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("SELECT count(*) FROM projects", [], |row| row.get(0))?)
}

/// Inserta o actualiza un proyecto descubierto, usando la ruta como clave.
///
/// Respeta la semántica de marcas de tiempo documentada en el módulo: el
/// escaneo de un proyecto sin cambios no toca `updated_at`.
pub fn upsert_by_path(
    conn: &Connection,
    discovered: &DiscoveredProject,
    now: i64,
) -> Result<UpsertOutcome> {
    let path = discovered
        .path
        .to_str()
        .ok_or_else(|| AppError::invalid_path(&discovered.path, "no es UTF-8 válido"))?;

    let Some(existing) = find_by_path(conn, path)? else {
        conn.execute(
            "INSERT INTO projects
                (name, path, is_git_repo, primary_language, missing, last_seen_at,
                 created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 0, ?5, ?5, ?5)",
            params![
                discovered.name,
                path,
                discovered.is_git_repo,
                discovered.primary_language,
                now
            ],
        )?;
        return Ok(UpsertOutcome {
            id: conn.last_insert_rowid(),
            created: true,
            metadata_changed: true,
        });
    };

    let metadata_changed = existing.name != discovered.name
        || existing.primary_language != discovered.primary_language
        || existing.is_git_repo != discovered.is_git_repo;

    if metadata_changed {
        conn.execute(
            "UPDATE projects
                SET name = ?2, is_git_repo = ?3, primary_language = ?4,
                    missing = 0, last_seen_at = ?5, updated_at = ?5
              WHERE id = ?1",
            params![
                existing.id,
                discovered.name,
                discovered.is_git_repo,
                discovered.primary_language,
                now
            ],
        )?;
    } else {
        conn.execute(
            "UPDATE projects SET missing = 0, last_seen_at = ?2 WHERE id = ?1",
            params![existing.id, now],
        )?;
    }

    Ok(UpsertOutcome {
        id: existing.id,
        created: false,
        metadata_changed,
    })
}

/// Marca como ausentes los proyectos bajo `roots` que el escaneo no ha visto.
///
/// `seen` contiene los identificadores que el escáner acaba de encontrar en
/// disco. Se comparan identificadores y no marcas de tiempo porque la
/// resolución de `last_seen_at` es de un segundo y dos escaneos consecutivos
/// pueden compartir instante.
///
/// Solo se consideran proyectos que cuelgan de alguna de las raíces recién
/// escaneadas: los que viven bajo una ruta deshabilitada o eliminada conservan
/// su estado. `updated_at` no se toca, porque `missing` no es un metadato
/// propio del proyecto sino una observación del escáner.
///
/// Devuelve cuántos proyectos han pasado a `missing = 1`.
pub fn mark_missing_under_roots(
    conn: &Connection,
    roots: &[std::path::PathBuf],
    seen: &HashSet<i64>,
) -> Result<usize> {
    if roots.is_empty() {
        return Ok(0);
    }

    let mut stmt = conn.prepare("SELECT id, path FROM projects WHERE missing = 0")?;
    let candidates: Vec<(i64, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let stale: Vec<i64> = candidates
        .into_iter()
        .filter(|(id, path)| {
            let path = Path::new(path);
            !seen.contains(id) && roots.iter().any(|root| path.starts_with(root))
        })
        .map(|(id, _)| id)
        .collect();

    for id in &stale {
        conn.execute("UPDATE projects SET missing = 1 WHERE id = ?1", params![id])?;
    }

    if !stale.is_empty() {
        tracing::info!(count = stale.len(), "proyectos marcados como ausentes");
    }
    Ok(stale.len())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::db::Db;

    fn discovered(path: &str, language: Option<&str>) -> DiscoveredProject {
        let path = PathBuf::from(path);
        DiscoveredProject {
            name: DiscoveredProject::name_from_path(&path),
            path,
            is_git_repo: true,
            primary_language: language.map(str::to_string),
            markers: vec![".git".into()],
        }
    }

    #[test]
    fn upsert_creates_project() {
        let db = Db::open_in_memory().unwrap();
        let project = discovered("/code/mosaic", Some("Rust"));

        let outcome = db
            .with_conn(|conn| upsert_by_path(conn, &project, 1_000))
            .unwrap();

        assert!(outcome.created);
        let stored = db.with_conn(|conn| get_by_id(conn, outcome.id)).unwrap();
        assert_eq!(stored.name, "mosaic");
        assert_eq!(stored.path, "/code/mosaic");
        assert_eq!(stored.primary_language.as_deref(), Some("Rust"));
        assert!(stored.is_git_repo);
        assert!(!stored.missing);
        assert_eq!(stored.created_at, 1_000);
        assert_eq!(stored.updated_at, 1_000);
        assert_eq!(stored.last_seen_at, Some(1_000));
    }

    #[test]
    fn upsert_does_not_duplicate_by_path() {
        let db = Db::open_in_memory().unwrap();
        let project = discovered("/code/mosaic", Some("Rust"));

        let first = db
            .with_conn(|conn| upsert_by_path(conn, &project, 1_000))
            .unwrap();
        let second = db
            .with_conn(|conn| upsert_by_path(conn, &project, 2_000))
            .unwrap();

        assert_eq!(first.id, second.id);
        assert!(!second.created);
        assert_eq!(db.with_conn(count).unwrap(), 1);
    }

    #[test]
    fn upsert_never_modifies_created_at() {
        let db = Db::open_in_memory().unwrap();
        let mut project = discovered("/code/mosaic", Some("Rust"));

        let outcome = db
            .with_conn(|conn| upsert_by_path(conn, &project, 1_000))
            .unwrap();

        project.primary_language = Some("TypeScript".into());
        db.with_conn(|conn| upsert_by_path(conn, &project, 9_999))
            .unwrap();

        let stored = db.with_conn(|conn| get_by_id(conn, outcome.id)).unwrap();
        assert_eq!(stored.created_at, 1_000, "created_at es inmutable");
    }

    #[test]
    fn upsert_advances_updated_at_only_when_metadata_changes() {
        let db = Db::open_in_memory().unwrap();
        let mut project = discovered("/code/mosaic", Some("Rust"));
        let id = db
            .with_conn(|conn| upsert_by_path(conn, &project, 1_000))
            .unwrap()
            .id;

        // Segundo escaneo sin cambios: updated_at se queda quieto.
        let unchanged = db
            .with_conn(|conn| upsert_by_path(conn, &project, 2_000))
            .unwrap();
        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();
        assert!(!unchanged.metadata_changed);
        assert_eq!(stored.updated_at, 1_000);

        // Cambia el lenguaje: updated_at avanza.
        project.primary_language = Some("TypeScript".into());
        let changed = db
            .with_conn(|conn| upsert_by_path(conn, &project, 3_000))
            .unwrap();
        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();
        assert!(changed.metadata_changed);
        assert_eq!(stored.updated_at, 3_000);

        // Cambia el nombre: updated_at avanza.
        project.name = "mosaic-renombrado".into();
        db.with_conn(|conn| upsert_by_path(conn, &project, 4_000))
            .unwrap();
        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();
        assert_eq!(stored.updated_at, 4_000);

        // Cambia is_git_repo: updated_at avanza.
        project.is_git_repo = false;
        db.with_conn(|conn| upsert_by_path(conn, &project, 5_000))
            .unwrap();
        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();
        assert_eq!(stored.updated_at, 5_000);
    }

    #[test]
    fn upsert_always_advances_last_seen_at() {
        let db = Db::open_in_memory().unwrap();
        let project = discovered("/code/mosaic", Some("Rust"));
        let id = db
            .with_conn(|conn| upsert_by_path(conn, &project, 1_000))
            .unwrap()
            .id;

        for ts in [2_000, 3_000, 4_000] {
            db.with_conn(|conn| upsert_by_path(conn, &project, ts))
                .unwrap();
            let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();
            assert_eq!(stored.last_seen_at, Some(ts));
            assert_eq!(stored.updated_at, 1_000, "sin cambios de metadatos");
        }
    }

    #[test]
    fn mark_missing_flags_projects_not_seen_in_the_last_scan() {
        let db = Db::open_in_memory().unwrap();
        let visto = discovered("/code/visto", None);
        let perdido = discovered("/code/perdido", None);

        db.with_conn(|conn| upsert_by_path(conn, &visto, 1_000))
            .unwrap();
        let perdido_id = db
            .with_conn(|conn| upsert_by_path(conn, &perdido, 1_000))
            .unwrap()
            .id;

        // Segundo escaneo: solo se vuelve a ver uno.
        let visto_id = db
            .with_conn(|conn| upsert_by_path(conn, &visto, 2_100))
            .unwrap()
            .id;
        let seen = HashSet::from([visto_id]);
        let flagged = db
            .with_conn(|conn| mark_missing_under_roots(conn, &[PathBuf::from("/code")], &seen))
            .unwrap();

        assert_eq!(flagged, 1);
        let perdido = db.with_conn(|conn| get_by_id(conn, perdido_id)).unwrap();
        assert!(perdido.missing);
        assert_eq!(perdido.updated_at, 1_000, "missing no toca updated_at");

        let visto = db
            .with_conn(|conn| find_by_path(conn, "/code/visto"))
            .unwrap()
            .unwrap();
        assert!(!visto.missing);
    }

    #[test]
    fn mark_missing_ignores_projects_outside_the_scanned_roots() {
        let db = Db::open_in_memory().unwrap();
        let fuera = discovered("/otro/sitio/app", None);
        let id = db
            .with_conn(|conn| upsert_by_path(conn, &fuera, 1_000))
            .unwrap()
            .id;

        let flagged = db
            .with_conn(|conn| {
                mark_missing_under_roots(conn, &[PathBuf::from("/code")], &HashSet::new())
            })
            .unwrap();

        assert_eq!(flagged, 0);
        assert!(!db.with_conn(|conn| get_by_id(conn, id)).unwrap().missing);
    }

    #[test]
    fn upsert_clears_missing_when_project_reappears() {
        let db = Db::open_in_memory().unwrap();
        let project = discovered("/code/mosaic", None);
        let id = db
            .with_conn(|conn| upsert_by_path(conn, &project, 1_000))
            .unwrap()
            .id;

        db.with_conn(|conn| {
            mark_missing_under_roots(conn, &[PathBuf::from("/code")], &HashSet::new())
        })
        .unwrap();
        assert!(db.with_conn(|conn| get_by_id(conn, id)).unwrap().missing);

        db.with_conn(|conn| upsert_by_path(conn, &project, 3_000))
            .unwrap();
        assert!(!db.with_conn(|conn| get_by_id(conn, id)).unwrap().missing);
    }

    #[test]
    fn list_all_puts_pinned_projects_first() {
        let db = Db::open_in_memory().unwrap();
        db.with_conn(|conn| upsert_by_path(conn, &discovered("/code/zeta", None), 1_000))
            .unwrap();
        let alfa = db
            .with_conn(|conn| upsert_by_path(conn, &discovered("/code/alfa", None), 1_000))
            .unwrap();
        db.with_conn(|conn| {
            conn.execute(
                "UPDATE projects SET pinned = 1 WHERE id = ?1",
                params![alfa.id],
            )?;
            Ok(())
        })
        .unwrap();

        let projects = db.with_conn(list_all).unwrap();
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].name, "alfa");
        assert_eq!(projects[1].name, "zeta");
    }

    #[test]
    fn get_by_id_reports_not_found() {
        let db = Db::open_in_memory().unwrap();
        let err = db.with_conn(|conn| get_by_id(conn, 42)).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
