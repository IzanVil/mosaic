//! Comandos de consulta de proyectos.

use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use crate::core::now_ts;
use crate::core::readme::{self, ReadmePreview};
use crate::db::repositories::projects::{self as repo, ProjectWithTags};
use crate::db::Db;
use crate::errors::CommandError;
use crate::model::project::normalize_notes;

use super::blocking;

/// Devuelve todos los proyectos con sus etiquetas y su estado Git: lo único que lee el tablero.
#[tauri::command]
pub async fn list_projects_with_tags(
    db: State<'_, Arc<Db>>,
) -> Result<Vec<ProjectWithTags>, CommandError> {
    let projects = db.with_conn(repo::list_all_with_tags)?;
    tracing::debug!(count = projects.len(), "list_projects_with_tags");
    Ok(projects)
}

/// Fija o quita la marca de favorito de un proyecto.
#[tauri::command]
pub async fn set_project_pinned(
    db: State<'_, Arc<Db>>,
    id: i64,
    pinned: bool,
) -> Result<(), CommandError> {
    db.with_conn(|conn| repo::set_pinned(conn, id, pinned))?;
    Ok(())
}

/// Guarda las notas de un proyecto y devuelve lo que ha quedado guardado.
#[tauri::command]
pub async fn set_project_notes(
    db: State<'_, Arc<Db>>,
    id: i64,
    notes: String,
) -> Result<Option<String>, CommandError> {
    let normalized = normalize_notes(&notes)?;
    let wrote = db.with_conn(|conn| repo::set_notes(conn, id, normalized.as_deref(), now_ts()))?;
    tracing::debug!(id, wrote, "set_project_notes");
    Ok(normalized)
}

/// Lee y renderiza el README; `null` si no tiene o si la carpeta ya no está.
#[tauri::command]
pub async fn get_project_readme(
    db: State<'_, Arc<Db>>,
    project_id: i64,
) -> Result<Option<ReadmePreview>, CommandError> {
    let project = db.with_conn(|conn| repo::get_by_id(conn, project_id))?;

    blocking("la lectura del README", move || {
        readme::read(&PathBuf::from(&project.path))
    })
    .await
}
