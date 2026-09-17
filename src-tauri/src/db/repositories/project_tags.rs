//! Repositorio de la relación N:M entre proyectos y etiquetas.
//!
//! Asignar no es un cambio de metadatos del proyecto: aquí nunca se escribe en
//! la tabla `projects`, así que `updated_at` no avanza al etiquetar. Lo fija el
//! test `assigning_tags_never_touches_the_projects_table`.

use std::collections::HashMap;

use rusqlite::{params, Connection};

use crate::core::tag::Tag;
use crate::errors::{AppError, Result};

/// Traduce una violación de clave ajena en un «no encontrado» legible.
///
/// `INSERT OR IGNORE` silencia los conflictos de unicidad, pero no los de
/// clave ajena: si el proyecto o la etiqueta no existen, SQLite falla.
fn map_missing_reference(err: rusqlite::Error, project_id: i64, tag_id: i64) -> AppError {
    match &err {
        rusqlite::Error::SqliteFailure(failure, _)
            if failure.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::NotFound(format!("proyecto {project_id} o etiqueta {tag_id}"))
        }
        _ => AppError::from(err),
    }
}

/// Asigna una etiqueta a un proyecto.
///
/// Es idempotente: repetir la asignación no falla ni duplica la fila.
pub fn assign(conn: &Connection, project_id: i64, tag_id: i64) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO project_tags (project_id, tag_id) VALUES (?1, ?2)",
        params![project_id, tag_id],
    )
    .map_err(|err| map_missing_reference(err, project_id, tag_id))?;
    Ok(())
}

/// Quita una etiqueta de un proyecto.
///
/// Quitar algo que no estaba asignado no es un error: el resultado que pedía
/// quien llama —que la etiqueta no esté— ya se cumple.
pub fn unassign(conn: &Connection, project_id: i64, tag_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM project_tags WHERE project_id = ?1 AND tag_id = ?2",
        params![project_id, tag_id],
    )?;
    Ok(())
}

/// Devuelve las etiquetas de un proyecto, ordenadas por nombre.
pub fn list_tags_for_project(conn: &Connection, project_id: i64) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, t.created_at
           FROM tags t
           JOIN project_tags pt ON pt.tag_id = t.id
          WHERE pt.project_id = ?1
          ORDER BY t.name COLLATE NOCASE ASC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Devuelve los identificadores de los proyectos que tienen una etiqueta.
pub fn list_projects_for_tag(conn: &Connection, tag_id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT project_id FROM project_tags WHERE tag_id = ?1")?;
    let rows = stmt.query_map(params![tag_id], |row| row.get(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Devuelve todas las asignaciones agrupadas por proyecto.
///
/// Es la segunda —y última— consulta de
/// [`crate::db::repositories::projects::list_all_with_tags`]: trae las
/// etiquetas de todos los proyectos de una vez para no hacer una consulta por
/// tarjeta del tablero.
pub fn list_all_grouped(conn: &Connection) -> Result<HashMap<i64, Vec<Tag>>> {
    let mut stmt = conn.prepare(
        "SELECT pt.project_id, t.id, t.name, t.color, t.created_at
           FROM project_tags pt
           JOIN tags t ON t.id = pt.tag_id
          ORDER BY pt.project_id, t.name COLLATE NOCASE ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            Tag {
                id: row.get(1)?,
                name: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
            },
        ))
    })?;

    let mut grouped: HashMap<i64, Vec<Tag>> = HashMap::new();
    for row in rows {
        let (project_id, tag) = row?;
        grouped.entry(project_id).or_default().push(tag);
    }
    Ok(grouped)
}

/// Asigna una etiqueta a varios proyectos y devuelve cuántas asignaciones son nuevas.
///
/// Preparado para la selección múltiple de la Fase 6. Va en una transacción
/// para que un identificador inválido no deje el lote a medias.
pub fn bulk_assign(conn: &mut Connection, project_ids: &[i64], tag_id: i64) -> Result<usize> {
    let tx = conn.transaction()?;
    let mut inserted = 0;
    {
        let mut stmt =
            tx.prepare("INSERT OR IGNORE INTO project_tags (project_id, tag_id) VALUES (?1, ?2)")?;
        for &project_id in project_ids {
            inserted += stmt
                .execute(params![project_id, tag_id])
                .map_err(|err| map_missing_reference(err, project_id, tag_id))?;
        }
    }
    tx.commit()?;
    Ok(inserted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repositories::{projects, tags};
    use crate::db::Db;

    fn project(db: &Db, path: &str) -> i64 {
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO projects (name, path, created_at, updated_at)
                 VALUES (?1, ?1, 100, 100)",
                params![path],
            )?;
            Ok(conn.last_insert_rowid())
        })
        .unwrap()
    }

    fn tag(db: &Db, name: &str) -> i64 {
        db.with_conn(|conn| tags::create(conn, name, "#3B82F6", 0))
            .unwrap()
            .id
    }

    #[test]
    fn assign_is_idempotent() {
        let db = Db::open_in_memory().unwrap();
        let project_id = project(&db, "/code/uno");
        let tag_id = tag(&db, "cliente");

        db.with_conn(|conn| assign(conn, project_id, tag_id))
            .unwrap();
        db.with_conn(|conn| assign(conn, project_id, tag_id))
            .unwrap();

        let assigned = db
            .with_conn(|conn| list_tags_for_project(conn, project_id))
            .unwrap();
        assert_eq!(
            assigned.len(),
            1,
            "la segunda asignación no duplica la fila"
        );
    }

    #[test]
    fn assign_reports_an_unknown_project_or_tag() {
        let db = Db::open_in_memory().unwrap();
        let project_id = project(&db, "/code/uno");
        let tag_id = tag(&db, "cliente");

        assert!(matches!(
            db.with_conn(|conn| assign(conn, 999, tag_id)),
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            db.with_conn(|conn| assign(conn, project_id, 999)),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn unassign_of_something_not_assigned_does_not_fail() {
        let db = Db::open_in_memory().unwrap();
        let project_id = project(&db, "/code/uno");
        let tag_id = tag(&db, "cliente");

        db.with_conn(|conn| unassign(conn, project_id, tag_id))
            .unwrap();
        db.with_conn(|conn| unassign(conn, 999, 999)).unwrap();
    }

    #[test]
    fn unassign_removes_only_that_pair() {
        let db = Db::open_in_memory().unwrap();
        let uno = project(&db, "/code/uno");
        let dos = project(&db, "/code/dos");
        let cliente = tag(&db, "cliente");
        let urgente = tag(&db, "urgente");

        for (p, t) in [(uno, cliente), (uno, urgente), (dos, cliente)] {
            db.with_conn(|conn| assign(conn, p, t)).unwrap();
        }
        db.with_conn(|conn| unassign(conn, uno, cliente)).unwrap();

        let de_uno: Vec<i64> = db
            .with_conn(|conn| list_tags_for_project(conn, uno))
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(de_uno, vec![urgente]);

        let de_dos: Vec<i64> = db
            .with_conn(|conn| list_tags_for_project(conn, dos))
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(de_dos, vec![cliente], "la otra asignación sigue intacta");
    }

    #[test]
    fn list_tags_for_project_returns_only_the_assigned_ones_sorted() {
        let db = Db::open_in_memory().unwrap();
        let project_id = project(&db, "/code/uno");
        let zeta = tag(&db, "zeta");
        let alfa = tag(&db, "Alfa");
        tag(&db, "sin-asignar");

        db.with_conn(|conn| assign(conn, project_id, zeta)).unwrap();
        db.with_conn(|conn| assign(conn, project_id, alfa)).unwrap();

        let names: Vec<String> = db
            .with_conn(|conn| list_tags_for_project(conn, project_id))
            .unwrap()
            .into_iter()
            .map(|t| t.name)
            .collect();
        assert_eq!(names, vec!["Alfa", "zeta"]);
    }

    #[test]
    fn list_tags_for_project_is_empty_when_nothing_is_assigned() {
        let db = Db::open_in_memory().unwrap();
        let project_id = project(&db, "/code/uno");
        assert!(db
            .with_conn(|conn| list_tags_for_project(conn, project_id))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn list_projects_for_tag_returns_the_ids() {
        let db = Db::open_in_memory().unwrap();
        let uno = project(&db, "/code/uno");
        let dos = project(&db, "/code/dos");
        project(&db, "/code/tres");
        let cliente = tag(&db, "cliente");

        db.with_conn(|conn| assign(conn, uno, cliente)).unwrap();
        db.with_conn(|conn| assign(conn, dos, cliente)).unwrap();

        let mut ids = db
            .with_conn(|conn| list_projects_for_tag(conn, cliente))
            .unwrap();
        ids.sort_unstable();
        assert_eq!(ids, vec![uno, dos]);
    }

    #[test]
    fn deleting_a_tag_keeps_the_project() {
        let db = Db::open_in_memory().unwrap();
        let project_id = project(&db, "/code/uno");
        let tag_id = tag(&db, "cliente");
        db.with_conn(|conn| assign(conn, project_id, tag_id))
            .unwrap();

        db.with_conn(|conn| tags::delete(conn, tag_id)).unwrap();

        let project = db
            .with_conn(|conn| projects::get_by_id(conn, project_id))
            .unwrap();
        assert_eq!(project.path, "/code/uno");
        assert!(db
            .with_conn(|conn| list_tags_for_project(conn, project_id))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn deleting_a_project_removes_its_assignments() {
        let db = Db::open_in_memory().unwrap();
        let uno = project(&db, "/code/uno");
        let dos = project(&db, "/code/dos");
        let cliente = tag(&db, "cliente");
        db.with_conn(|conn| assign(conn, uno, cliente)).unwrap();
        db.with_conn(|conn| assign(conn, dos, cliente)).unwrap();

        // Mosaic nunca borra proyectos (los marca `missing`), pero si algún día
        // lo hiciera, el CASCADE no puede dejar asignaciones huérfanas.
        db.with_conn(|conn| {
            conn.execute("DELETE FROM projects WHERE id = ?1", params![uno])?;
            Ok(())
        })
        .unwrap();

        assert_eq!(
            db.with_conn(|conn| list_projects_for_tag(conn, cliente))
                .unwrap(),
            vec![dos]
        );
    }

    #[test]
    fn assigning_tags_never_touches_the_projects_table() {
        let db = Db::open_in_memory().unwrap();
        let project_id = project(&db, "/code/uno");
        let tag_id = tag(&db, "cliente");
        let before = db
            .with_conn(|conn| projects::get_by_id(conn, project_id))
            .unwrap();

        db.with_conn(|conn| assign(conn, project_id, tag_id))
            .unwrap();
        db.with_conn(|conn| unassign(conn, project_id, tag_id))
            .unwrap();

        let after = db
            .with_conn(|conn| projects::get_by_id(conn, project_id))
            .unwrap();
        assert_eq!(
            after.updated_at, before.updated_at,
            "etiquetar no es un cambio de metadatos del proyecto"
        );
        assert_eq!(after.last_seen_at, before.last_seen_at);
        assert_eq!(after.created_at, before.created_at);
        assert_eq!(after.missing, before.missing);
    }

    #[test]
    fn list_all_grouped_groups_by_project() {
        let db = Db::open_in_memory().unwrap();
        let uno = project(&db, "/code/uno");
        let dos = project(&db, "/code/dos");
        project(&db, "/code/sin-etiquetas");
        let zeta = tag(&db, "zeta");
        let alfa = tag(&db, "Alfa");

        for (p, t) in [(uno, zeta), (uno, alfa), (dos, alfa)] {
            db.with_conn(|conn| assign(conn, p, t)).unwrap();
        }

        let grouped = db.with_conn(list_all_grouped).unwrap();
        assert_eq!(
            grouped.len(),
            2,
            "un proyecto sin etiquetas no tiene entrada"
        );
        let de_uno: Vec<&str> = grouped[&uno].iter().map(|t| t.name.as_str()).collect();
        assert_eq!(de_uno, vec!["Alfa", "zeta"], "ordenadas por nombre");
        assert_eq!(grouped[&dos].len(), 1);
    }

    #[test]
    fn bulk_assign_counts_only_the_new_rows() {
        let db = Db::open_in_memory().unwrap();
        let uno = project(&db, "/code/uno");
        let dos = project(&db, "/code/dos");
        let cliente = tag(&db, "cliente");
        db.with_conn(|conn| assign(conn, uno, cliente)).unwrap();

        let inserted = db
            .with_conn_mut(|conn| bulk_assign(conn, &[uno, dos], cliente))
            .unwrap();
        assert_eq!(inserted, 1, "«uno» ya la tenía");
    }

    #[test]
    fn bulk_assign_rolls_back_on_an_unknown_project() {
        let db = Db::open_in_memory().unwrap();
        let uno = project(&db, "/code/uno");
        let cliente = tag(&db, "cliente");

        let err = db.with_conn_mut(|conn| bulk_assign(conn, &[uno, 999], cliente));
        assert!(matches!(err, Err(AppError::NotFound(_))));
        assert!(
            db.with_conn(|conn| list_projects_for_tag(conn, cliente))
                .unwrap()
                .is_empty(),
            "el lote no debe quedar a medias"
        );
    }
}
