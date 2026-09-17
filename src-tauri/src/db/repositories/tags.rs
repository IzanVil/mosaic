use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::core::tag::{self, Tag, TagWithCount};
use crate::errors::{AppError, Result};

const COLUMNS: &str = "id, name, color, created_at";

fn row_to_tag(row: &Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        created_at: row.get(3)?,
    })
}

fn map_name_conflict(err: rusqlite::Error, name: &str) -> AppError {
    match &err {
        rusqlite::Error::SqliteFailure(failure, _)
            if failure.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::Validation(format!("ya existe una etiqueta llamada «{name}»"))
        }
        _ => AppError::from(err),
    }
}

pub fn get_by_id(conn: &Connection, id: i64) -> Result<Tag> {
    let sql = format!("SELECT {COLUMNS} FROM tags WHERE id = ?1");
    conn.query_row(&sql, params![id], row_to_tag)
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("etiqueta {id}")))
}

pub fn get_by_name_ci(conn: &Connection, name: &str) -> Result<Option<Tag>> {
    let sql = format!("SELECT {COLUMNS} FROM tags WHERE lower(name) = ?1");
    Ok(conn
        .query_row(&sql, params![tag::name_key(name)], row_to_tag)
        .optional()?)
}

pub fn create(conn: &Connection, name: &str, color: &str, now: i64) -> Result<Tag> {
    let name = tag::validate_name(name)?;
    let color = tag::normalize_color(color)?;

    if let Some(existing) = get_by_name_ci(conn, &name)? {
        return Err(AppError::Validation(format!(
            "ya existe una etiqueta llamada «{}»",
            existing.name
        )));
    }

    conn.execute(
        "INSERT INTO tags (name, color, created_at) VALUES (?1, ?2, ?3)",
        params![name, color, now],
    )
    .map_err(|err| map_name_conflict(err, &name))?;

    let tag = Tag {
        id: conn.last_insert_rowid(),
        name,
        color,
        created_at: now,
    };
    tracing::debug!(id = tag.id, name = %tag.name, "etiqueta creada");
    Ok(tag)
}

pub fn list_all(conn: &Connection) -> Result<Vec<TagWithCount>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, t.created_at, count(pt.project_id)
           FROM tags t
           LEFT JOIN project_tags pt ON pt.tag_id = t.id
          GROUP BY t.id
          ORDER BY t.name COLLATE NOCASE ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(TagWithCount {
            tag: row_to_tag(row)?,
            project_count: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn update(conn: &Connection, id: i64, name: &str, color: &str) -> Result<Tag> {
    let name = tag::validate_name(name)?;
    let color = tag::normalize_color(color)?;

    if let Some(existing) = get_by_name_ci(conn, &name)? {
        if existing.id != id {
            return Err(AppError::Validation(format!(
                "ya existe una etiqueta llamada «{}»",
                existing.name
            )));
        }
    }

    let affected = conn
        .execute(
            "UPDATE tags SET name = ?2, color = ?3 WHERE id = ?1",
            params![id, name, color],
        )
        .map_err(|err| map_name_conflict(err, &name))?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("etiqueta {id}")));
    }

    get_by_id(conn, id)
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let affected = conn.execute("DELETE FROM tags WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("etiqueta {id}")));
    }
    tracing::debug!(id, "etiqueta borrada");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tag::MAX_NAME_LEN;
    use crate::db::repositories::project_tags;
    use crate::db::Db;

    fn project(db: &Db, path: &str) -> i64 {
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO projects (name, path, created_at, updated_at)
                 VALUES (?1, ?1, 0, 0)",
                params![path],
            )?;
            Ok(conn.last_insert_rowid())
        })
        .unwrap()
    }

    #[test]
    fn create_stores_a_normalized_tag() {
        let db = Db::open_in_memory().unwrap();

        let tag = db
            .with_conn(|conn| create(conn, "  Cliente  ", "#3b82f6", 1_000))
            .unwrap();

        assert_eq!(tag.name, "Cliente", "el nombre se recorta pero no se baja");
        assert_eq!(tag.color, "#3B82F6", "el color se guarda en mayúsculas");
        assert_eq!(tag.created_at, 1_000);
        assert!(tag.id > 0);
    }

    #[test]
    fn create_rejects_an_empty_name() {
        let db = Db::open_in_memory().unwrap();
        let err = db.with_conn(|conn| create(conn, "   ", "#3B82F6", 0));
        assert!(matches!(err, Err(AppError::Validation(_))));
    }

    #[test]
    fn create_rejects_a_name_over_the_limit() {
        let db = Db::open_in_memory().unwrap();
        let name = "a".repeat(MAX_NAME_LEN + 1);
        let err = db.with_conn(|conn| create(conn, &name, "#3B82F6", 0));
        assert!(matches!(err, Err(AppError::Validation(_))));
    }

    #[test]
    fn create_rejects_a_duplicate_name_ignoring_case() {
        let db = Db::open_in_memory().unwrap();
        db.with_conn(|conn| create(conn, "Cliente", "#3B82F6", 0))
            .unwrap();

        let err = db.with_conn(|conn| create(conn, "  cliente ", "#EF4444", 0));
        assert!(
            matches!(err, Err(AppError::Validation(_))),
            "«cliente» y «Cliente» son la misma etiqueta"
        );

        let count = db.with_conn(list_all).unwrap().len();
        assert_eq!(count, 1);
    }

    #[test]
    fn create_rejects_an_invalid_color() {
        let db = Db::open_in_memory().unwrap();
        let err = db.with_conn(|conn| create(conn, "cliente", "azul", 0));
        assert!(matches!(err, Err(AppError::Validation(_))));
    }

    #[test]
    fn update_changes_name_and_color() {
        let db = Db::open_in_memory().unwrap();
        let tag = db
            .with_conn(|conn| create(conn, "cliente", "#3B82F6", 0))
            .unwrap();

        let updated = db
            .with_conn(|conn| update(conn, tag.id, " Clientes ", "#ef4444"))
            .unwrap();

        assert_eq!(updated.name, "Clientes");
        assert_eq!(updated.color, "#EF4444");
        assert_eq!(updated.created_at, tag.created_at, "created_at no se toca");
    }

    #[test]
    fn update_allows_changing_only_the_casing_of_its_own_name() {
        let db = Db::open_in_memory().unwrap();
        let tag = db
            .with_conn(|conn| create(conn, "cliente", "#3B82F6", 0))
            .unwrap();

        let updated = db
            .with_conn(|conn| update(conn, tag.id, "Cliente", "#3B82F6"))
            .unwrap();
        assert_eq!(updated.name, "Cliente");
    }

    #[test]
    fn update_rejects_taking_the_name_of_another_tag() {
        let db = Db::open_in_memory().unwrap();
        db.with_conn(|conn| create(conn, "cliente", "#3B82F6", 0))
            .unwrap();
        let personal = db
            .with_conn(|conn| create(conn, "personal", "#EF4444", 0))
            .unwrap();

        let err = db.with_conn(|conn| update(conn, personal.id, "CLIENTE", "#EF4444"));
        assert!(matches!(err, Err(AppError::Validation(_))));
    }

    #[test]
    fn update_and_delete_report_a_missing_tag() {
        let db = Db::open_in_memory().unwrap();
        assert!(matches!(
            db.with_conn(|conn| update(conn, 42, "cliente", "#3B82F6")),
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            db.with_conn(|conn| delete(conn, 42)),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn delete_removes_the_tag() {
        let db = Db::open_in_memory().unwrap();
        let tag = db
            .with_conn(|conn| create(conn, "cliente", "#3B82F6", 0))
            .unwrap();

        db.with_conn(|conn| delete(conn, tag.id)).unwrap();
        assert!(db.with_conn(list_all).unwrap().is_empty());
    }

    #[test]
    fn delete_cascades_into_project_tags() {
        let db = Db::open_in_memory().unwrap();
        let tag = db
            .with_conn(|conn| create(conn, "cliente", "#3B82F6", 0))
            .unwrap();
        let project_id = project(&db, "/code/uno");
        db.with_conn(|conn| project_tags::assign(conn, project_id, tag.id))
            .unwrap();

        db.with_conn(|conn| delete(conn, tag.id)).unwrap();

        let rows: i64 = db
            .with_conn(|conn| {
                Ok(conn.query_row("SELECT count(*) FROM project_tags", [], |row| row.get(0))?)
            })
            .unwrap();
        assert_eq!(rows, 0, "el CASCADE debe limpiar las asignaciones");
    }

    #[test]
    fn list_all_counts_the_projects_of_each_tag() {
        let db = Db::open_in_memory().unwrap();
        let cliente = db
            .with_conn(|conn| create(conn, "cliente", "#3B82F6", 0))
            .unwrap();
        let vacia = db
            .with_conn(|conn| create(conn, "archivado", "#EF4444", 0))
            .unwrap();
        let uno = project(&db, "/code/uno");
        let dos = project(&db, "/code/dos");

        db.with_conn(|conn| project_tags::assign(conn, uno, cliente.id))
            .unwrap();
        db.with_conn(|conn| project_tags::assign(conn, dos, cliente.id))
            .unwrap();

        let tags = db.with_conn(list_all).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].tag.id, vacia.id);
        assert_eq!(tags[0].project_count, 0, "una etiqueta sin usar sale con 0");
        assert_eq!(tags[1].tag.id, cliente.id);
        assert_eq!(tags[1].project_count, 2);
    }

    #[test]
    fn list_all_orders_by_name_ignoring_case() {
        let db = Db::open_in_memory().unwrap();
        for name in ["zeta", "Alfa", "beta"] {
            db.with_conn(|conn| create(conn, name, "#3B82F6", 0))
                .unwrap();
        }

        let names: Vec<String> = db
            .with_conn(list_all)
            .unwrap()
            .into_iter()
            .map(|t| t.tag.name)
            .collect();
        assert_eq!(names, vec!["Alfa", "beta", "zeta"]);
    }

    #[test]
    fn get_by_name_ci_finds_a_tag_in_any_casing() {
        let db = Db::open_in_memory().unwrap();
        let tag = db
            .with_conn(|conn| create(conn, "Cliente", "#3B82F6", 0))
            .unwrap();

        for query in ["Cliente", "cliente", "CLIENTE", "  cliente  "] {
            let found = db.with_conn(|conn| get_by_name_ci(conn, query)).unwrap();
            assert_eq!(found.map(|t| t.id), Some(tag.id), "búsqueda «{query}»");
        }

        assert!(db
            .with_conn(|conn| get_by_name_ci(conn, "otra"))
            .unwrap()
            .is_none());
    }
}
