//! Repositorio de proyectos.
//!
//! Marcas de tiempo:
//!
//! - `created_at` se fija al insertar y no cambia nunca.
//! - `updated_at` solo avanza si cambian columnas propias, y no hay más caminos:
//!   [`upsert_by_path`] compara `name`, `primary_language` e `is_git_repo`, y
//!   [`set_notes`] la mueve si las notas cambian de verdad. Fijar y abrir no la
//!   tocan: son preferencias de presentación, no metadatos de la carpeta.
//! - `last_seen_at` avanza cada vez que el escáner ve el proyecto, y `missing` es
//!   `0` si lo ve y `1` si no. Tampoco mueven `updated_at`: son observaciones.
//!
//! `is_git_repo` es la columna de `projects`, no el estado vivo de
//! `git_status_cache`: el refresco de Git solo escribe en la caché y no puede
//! mover `updated_at` (`refreshing_git_status_never_touches_the_projects_table`).
//!
//! Que un proyecto desaparezca del disco no borra nada: sus etiquetas y notas siguen.

use std::collections::HashSet;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::db::repositories::git_status::GitStatusEntry;
use crate::db::repositories::project_tags;
use crate::errors::{AppError, Result};
use crate::model::git::GitStatus;
use crate::model::project::{DiscoveredProject, Project};
use crate::model::tag::Tag;

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
        .ok_or_else(|| {
            AppError::not_found("project_not_found", format!("proyecto {id}")).with("id", id)
        })
}

/// Devuelve todos los proyectos: primero los fijados, luego por nombre.
pub fn list_all(conn: &Connection) -> Result<Vec<Project>> {
    let sql =
        format!("SELECT {COLUMNS} FROM projects ORDER BY pinned DESC, name COLLATE NOCASE ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_project)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Proyecto con lo que necesita una tarjeta, aplanado como `ProjectWithTags` en el frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectWithTags {
    #[serde(flatten)]
    pub project: Project,
    /// Etiquetas asignadas, ordenadas por nombre. Vacío si no tiene ninguna.
    pub tags: Vec<Tag>,
    /// Estado Git cacheado, o `None` si no hay repositorio o nunca se refrescó.
    pub git_status: Option<GitStatusEntry>,
}

/// Columnas de `git_status_cache` para el `LEFT JOIN`; no chocan con las de [`COLUMNS`].
const GIT_COLUMNS: &str = "branch, ahead, behind, is_dirty, last_commit_sha,
     last_commit_msg, last_commit_at, remote_url, refreshed_at";

/// Todos los proyectos, ausentes incluidos, con sus etiquetas y su estado Git.
pub fn list_all_with_tags(conn: &Connection) -> Result<Vec<ProjectWithTags>> {
    let sql = format!(
        "SELECT {COLUMNS}, {GIT_COLUMNS}
           FROM projects
           LEFT JOIN git_status_cache ON git_status_cache.project_id = projects.id
          ORDER BY pinned DESC, name COLLATE NOCASE ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        let project = row_to_project(row)?;
        // `refreshed_at` es NOT NULL en la caché: vacío solo si el LEFT JOIN no encontró fila.
        let refreshed_at: Option<i64> = row.get(20)?;
        let git_status = match refreshed_at {
            Some(refreshed_at) => Some(GitStatusEntry {
                project_id: project.id,
                refreshed_at,
                status: GitStatus {
                    branch: row.get(12)?,
                    ahead: row.get(13)?,
                    behind: row.get(14)?,
                    is_dirty: row.get(15)?,
                    last_commit_sha: row.get(16)?,
                    last_commit_msg: row.get(17)?,
                    last_commit_at: row.get(18)?,
                    remote_url: row.get(19)?,
                },
            }),
            None => None,
        };
        Ok((project, git_status))
    })?;
    let base = rows.collect::<rusqlite::Result<Vec<_>>>()?;

    // Dos consultas en total: una por proyecto haría 501 viajes a SQLite con 500 proyectos.
    let mut grouped = project_tags::list_all_grouped(conn)?;

    Ok(base
        .into_iter()
        .map(|(project, git_status)| ProjectWithTags {
            tags: grouped.remove(&project.id).unwrap_or_default(),
            project,
            git_status,
        })
        .collect())
}

/// Número total de proyectos registrados.
pub fn count(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("SELECT count(*) FROM projects", [], |row| row.get(0))?)
}

/// Inserta o actualiza un proyecto descubierto, con la ruta como clave.
pub fn upsert_by_path(
    conn: &Connection,
    discovered: &DiscoveredProject,
    now: i64,
) -> Result<UpsertOutcome> {
    let path = discovered.path.to_str().ok_or_else(|| {
        AppError::invalid_path(&discovered.path, "path_not_utf8", "no es UTF-8 válido")
    })?;

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

/// Fija o quita la marca de favorito, sin tocar `updated_at`.
pub fn set_pinned(conn: &Connection, id: i64, pinned: bool) -> Result<()> {
    let affected = conn.execute(
        "UPDATE projects SET pinned = ?2 WHERE id = ?1",
        params![id, pinned],
    )?;
    if affected == 0 {
        return Err(
            AppError::not_found("project_not_found", format!("proyecto {id}")).with("id", id),
        );
    }
    Ok(())
}

/// Guarda las notas, ya normalizadas, y dice si ha escrito; `None` las borra.
pub fn set_notes(conn: &Connection, id: i64, notes: Option<&str>, now: i64) -> Result<bool> {
    let current: Option<String> = conn
        .query_row(
            "SELECT notes FROM projects WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| {
            AppError::not_found("project_not_found", format!("proyecto {id}")).with("id", id)
        })?;

    // El editor autoguarda en cada pausa: sin esto, `updated_at` avanzaría sin cambiar nada.
    if current.as_deref() == notes {
        return Ok(false);
    }

    conn.execute(
        "UPDATE projects SET notes = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, notes, now],
    )?;
    Ok(true)
}

/// Apunta que el proyecto se acaba de abrir, sin tocar `updated_at`.
pub fn touch_last_opened(conn: &Connection, id: i64, now: i64) -> Result<()> {
    let affected = conn.execute(
        "UPDATE projects SET last_opened_at = ?2 WHERE id = ?1",
        params![id, now],
    )?;
    if affected == 0 {
        return Err(
            AppError::not_found("project_not_found", format!("proyecto {id}")).with("id", id),
        );
    }
    Ok(())
}

/// Marca ausentes los proyectos bajo `roots` que no están en `seen`, y dice cuántos.
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

    // Identificadores y no `last_seen_at`: con resolución de un segundo, dos escaneos
    // seguidos comparten instante. Lo de una ruta deshabilitada o borrada no se toca.
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

        let unchanged = db
            .with_conn(|conn| upsert_by_path(conn, &project, 2_000))
            .unwrap();
        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();
        assert!(!unchanged.metadata_changed);
        assert_eq!(stored.updated_at, 1_000);

        project.primary_language = Some("TypeScript".into());
        let changed = db
            .with_conn(|conn| upsert_by_path(conn, &project, 3_000))
            .unwrap();
        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();
        assert!(changed.metadata_changed);
        assert_eq!(stored.updated_at, 3_000);

        project.name = "mosaic-renombrado".into();
        db.with_conn(|conn| upsert_by_path(conn, &project, 4_000))
            .unwrap();
        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();
        assert_eq!(stored.updated_at, 4_000);

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
    fn set_pinned_toggles_the_flag_without_touching_updated_at() {
        let db = Db::open_in_memory().unwrap();
        let id = db
            .with_conn(|conn| upsert_by_path(conn, &discovered("/code/mosaic", None), 1_000))
            .unwrap()
            .id;

        db.with_conn(|conn| set_pinned(conn, id, true)).unwrap();
        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();
        assert!(stored.pinned);
        assert_eq!(
            stored.updated_at, 1_000,
            "fijar no es un cambio de metadatos"
        );

        db.with_conn(|conn| set_pinned(conn, id, false)).unwrap();
        assert!(!db.with_conn(|conn| get_by_id(conn, id)).unwrap().pinned);
    }

    #[test]
    fn set_notes_saves_the_text_and_advances_updated_at() {
        let db = Db::open_in_memory().unwrap();
        let id = db
            .with_conn(|conn| upsert_by_path(conn, &discovered("/code/mosaic", None), 1_000))
            .unwrap()
            .id;

        let wrote = db
            .with_conn(|conn| set_notes(conn, id, Some("pendiente: firmar el .dmg"), 2_000))
            .unwrap();
        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();

        assert!(wrote);
        assert_eq!(stored.notes.as_deref(), Some("pendiente: firmar el .dmg"));
        assert_eq!(stored.updated_at, 2_000, "las notas son metadatos propios");
    }

    #[test]
    fn set_notes_with_none_stores_null() {
        let db = Db::open_in_memory().unwrap();
        let id = db
            .with_conn(|conn| upsert_by_path(conn, &discovered("/code/mosaic", None), 1_000))
            .unwrap()
            .id;
        db.with_conn(|conn| set_notes(conn, id, Some("algo"), 2_000))
            .unwrap();

        let wrote = db
            .with_conn(|conn| set_notes(conn, id, None, 3_000))
            .unwrap();
        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();

        assert!(wrote);
        assert_eq!(stored.notes, None);
        assert_eq!(stored.updated_at, 3_000, "borrarlas también es un cambio");
    }

    #[test]
    fn set_notes_does_not_write_when_the_text_is_the_same() {
        let db = Db::open_in_memory().unwrap();
        let id = db
            .with_conn(|conn| upsert_by_path(conn, &discovered("/code/mosaic", None), 1_000))
            .unwrap()
            .id;
        db.with_conn(|conn| set_notes(conn, id, Some("igual"), 2_000))
            .unwrap();

        let wrote = db
            .with_conn(|conn| set_notes(conn, id, Some("igual"), 9_000))
            .unwrap();
        let empty_again = db
            .with_conn(|conn| {
                set_notes(conn, id, None, 9_500)?;
                set_notes(conn, id, None, 9_900)
            })
            .unwrap();
        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();

        assert!(!wrote, "el mismo texto no se vuelve a escribir");
        assert!(!empty_again, "NULL sobre NULL tampoco");
        assert_eq!(stored.updated_at, 9_500);
    }

    #[test]
    fn set_notes_touches_nothing_else_and_rejects_unknown_projects() {
        let db = Db::open_in_memory().unwrap();
        let id = db
            .with_conn(|conn| upsert_by_path(conn, &discovered("/code/mosaic", None), 1_000))
            .unwrap()
            .id;
        db.with_conn(|conn| {
            set_pinned(conn, id, true)?;
            touch_last_opened(conn, id, 1_500)
        })
        .unwrap();
        let before = db.with_conn(|conn| get_by_id(conn, id)).unwrap();

        db.with_conn(|conn| set_notes(conn, id, Some("nota"), 2_000))
            .unwrap();
        let after = db.with_conn(|conn| get_by_id(conn, id)).unwrap();

        assert_eq!(after.created_at, before.created_at);
        assert_eq!(after.last_seen_at, before.last_seen_at);
        assert_eq!(after.last_opened_at, before.last_opened_at);
        assert_eq!(after.missing, before.missing);
        assert_eq!(after.pinned, before.pinned);
        assert_eq!(after.name, before.name);

        let missing = db.with_conn(|conn| set_notes(conn, id + 999, Some("x"), 2_000));
        assert!(matches!(missing, Err(AppError::NotFound(_))));
    }

    #[test]
    fn touch_last_opened_records_the_moment_without_touching_updated_at() {
        let db = Db::open_in_memory().unwrap();
        let id = db
            .with_conn(|conn| upsert_by_path(conn, &discovered("/code/mosaic", None), 1_000))
            .unwrap()
            .id;
        assert_eq!(
            db.with_conn(|conn| get_by_id(conn, id))
                .unwrap()
                .last_opened_at,
            None
        );

        db.with_conn(|conn| touch_last_opened(conn, id, 7_000))
            .unwrap();

        let stored = db.with_conn(|conn| get_by_id(conn, id)).unwrap();
        assert_eq!(stored.last_opened_at, Some(7_000));
        assert_eq!(stored.updated_at, 1_000);
    }

    #[test]
    fn pinning_and_opening_report_missing_projects() {
        let db = Db::open_in_memory().unwrap();

        assert!(matches!(
            db.with_conn(|conn| set_pinned(conn, 42, true)).unwrap_err(),
            AppError::NotFound(_)
        ));
        assert!(matches!(
            db.with_conn(|conn| touch_last_opened(conn, 42, 1))
                .unwrap_err(),
            AppError::NotFound(_)
        ));
    }

    #[test]
    fn get_by_id_reports_not_found() {
        let db = Db::open_in_memory().unwrap();
        let err = db.with_conn(|conn| get_by_id(conn, 42)).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn list_all_with_tags_returns_an_empty_vector_for_untagged_projects() {
        let db = Db::open_in_memory().unwrap();
        db.with_conn(|conn| upsert_by_path(conn, &discovered("/code/uno", None), 1))
            .unwrap();

        let rows = db.with_conn(list_all_with_tags).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(
            rows[0].tags.is_empty(),
            "sin etiquetas debe ser una lista vacía, no ausente"
        );
        assert!(
            rows[0].git_status.is_none(),
            "sin fila en la caché el estado Git es null"
        );
    }

    #[test]
    fn list_all_with_tags_attaches_tags_and_cached_git_status() {
        use crate::db::repositories::{git_status, project_tags, tags};
        use crate::model::git::GitStatus;

        let db = Db::open_in_memory().unwrap();
        let uno = db
            .with_conn(|conn| upsert_by_path(conn, &discovered("/code/uno", None), 1))
            .unwrap()
            .id;
        db.with_conn(|conn| upsert_by_path(conn, &discovered("/code/dos", None), 1))
            .unwrap();

        let cliente = db
            .with_conn(|conn| tags::create(conn, "cliente", "#3B82F6", 0))
            .unwrap();
        db.with_conn(|conn| project_tags::assign(conn, uno, cliente.id))
            .unwrap();

        let status = GitStatus {
            branch: Some("main".into()),
            ahead: Some(2),
            behind: None,
            is_dirty: true,
            last_commit_sha: Some("abc1234".into()),
            last_commit_msg: Some("mensaje".into()),
            last_commit_at: Some(500),
            remote_url: None,
        };
        db.with_conn(|conn| git_status::upsert(conn, uno, &status, 900))
            .unwrap();

        let rows = db.with_conn(list_all_with_tags).unwrap();
        let con_etiqueta = rows.iter().find(|r| r.project.id == uno).unwrap();
        let sin_etiqueta = rows.iter().find(|r| r.project.id != uno).unwrap();

        assert_eq!(con_etiqueta.tags, vec![cliente]);
        let entry = con_etiqueta.git_status.as_ref().unwrap();
        assert_eq!(entry.project_id, uno);
        assert_eq!(entry.refreshed_at, 900);
        assert_eq!(
            entry.status, status,
            "el estado llega completo, sin recortes"
        );

        assert!(sin_etiqueta.tags.is_empty());
        assert!(sin_etiqueta.git_status.is_none());
    }

    #[test]
    fn list_all_with_tags_keeps_the_same_order_as_list_all() {
        let db = Db::open_in_memory().unwrap();
        for path in ["/code/zeta", "/code/alfa", "/code/beta"] {
            db.with_conn(|conn| upsert_by_path(conn, &discovered(path, None), 1))
                .unwrap();
        }
        let beta = db
            .with_conn(|conn| find_by_path(conn, "/code/beta"))
            .unwrap()
            .unwrap();
        db.with_conn(|conn| set_pinned(conn, beta.id, true))
            .unwrap();

        let esperado: Vec<i64> = db
            .with_conn(list_all)
            .unwrap()
            .into_iter()
            .map(|p| p.id)
            .collect();
        let obtenido: Vec<i64> = db
            .with_conn(list_all_with_tags)
            .unwrap()
            .into_iter()
            .map(|r| r.project.id)
            .collect();

        assert_eq!(obtenido, esperado, "fijados primero y luego por nombre");
    }

    #[test]
    fn list_all_with_tags_includes_missing_projects() {
        let db = Db::open_in_memory().unwrap();
        db.with_conn(|conn| upsert_by_path(conn, &discovered("/code/uno", None), 1))
            .unwrap();
        db.with_conn(|conn| {
            mark_missing_under_roots(conn, &[PathBuf::from("/code")], &HashSet::new())
        })
        .unwrap();

        let rows = db.with_conn(list_all_with_tags).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(
            rows[0].project.missing,
            "filtrar ausentes es decisión de la vista, no del repositorio"
        );
    }
}
