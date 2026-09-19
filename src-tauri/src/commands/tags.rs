//! Comandos de gestión de etiquetas.
//!
//! Capa fina: toda la validación vive en [`crate::core::tag`] y todo el SQL en
//! los repositorios. Aquí solo se resuelve el instante actual y se convierte el
//! error al `String` que espera el puente de Tauri.

use std::sync::Arc;

use tauri::State;

use crate::core::now_ts;
use crate::core::tag::{Tag, TagWithCount};
use crate::db::repositories::{project_tags, tags as repo};
use crate::db::Db;

/// Crea una etiqueta. El nombre se recorta y el color se normaliza a `#RRGGBB`.
#[tauri::command]
pub async fn create_tag(
    db: State<'_, Arc<Db>>,
    name: String,
    color: String,
) -> Result<Tag, String> {
    let tag = db.with_conn(|conn| repo::create(conn, &name, &color, now_ts()))?;
    tracing::debug!(id = tag.id, name = %tag.name, "create_tag");
    Ok(tag)
}

/// Devuelve todas las etiquetas con su número de proyectos.
#[tauri::command]
pub async fn list_tags(db: State<'_, Arc<Db>>) -> Result<Vec<TagWithCount>, String> {
    let tags = db.with_conn(repo::list_all)?;
    tracing::debug!(count = tags.len(), "list_tags");
    Ok(tags)
}

/// Renombra una etiqueta y/o le cambia el color.
#[tauri::command]
pub async fn update_tag(
    db: State<'_, Arc<Db>>,
    id: i64,
    name: String,
    color: String,
) -> Result<Tag, String> {
    Ok(db.with_conn(|conn| repo::update(conn, id, &name, &color))?)
}

/// Borra una etiqueta y, por cascada, sus asignaciones.
#[tauri::command]
pub async fn delete_tag(db: State<'_, Arc<Db>>, id: i64) -> Result<(), String> {
    db.with_conn(|conn| repo::delete(conn, id))?;
    Ok(())
}

/// Asigna una etiqueta a un proyecto. Repetirlo no falla.
#[tauri::command]
pub async fn assign_tag(
    db: State<'_, Arc<Db>>,
    project_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    db.with_conn(|conn| project_tags::assign(conn, project_id, tag_id))?;
    Ok(())
}

/// Quita una etiqueta de un proyecto. Quitar algo no asignado no falla.
#[tauri::command]
pub async fn unassign_tag(
    db: State<'_, Arc<Db>>,
    project_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    db.with_conn(|conn| project_tags::unassign(conn, project_id, tag_id))?;
    Ok(())
}

/// Devuelve las etiquetas de un proyecto, ordenadas por nombre.
#[tauri::command]
pub async fn list_tags_for_project(
    db: State<'_, Arc<Db>>,
    project_id: i64,
) -> Result<Vec<Tag>, String> {
    Ok(db.with_conn(|conn| project_tags::list_tags_for_project(conn, project_id))?)
}
