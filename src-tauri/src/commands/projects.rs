//! Comandos de consulta de proyectos.

use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use crate::core::project::normalize_notes;
use crate::core::readme::{self, ReadmePreview};
use crate::core::{now_ts, Project};
use crate::db::repositories::projects::{self as repo, ProjectWithTags};
use crate::db::Db;
use crate::errors::AppError;

/// Devuelve todos los proyectos: primero los fijados, luego por nombre.
#[tauri::command]
pub async fn list_projects(db: State<'_, Arc<Db>>) -> Result<Vec<Project>, String> {
    let projects = db.with_conn(repo::list_all)?;
    tracing::debug!(count = projects.len(), "list_projects");
    Ok(projects)
}

/// Devuelve todos los proyectos con sus etiquetas y su estado Git cacheado.
///
/// Es la única lectura que necesita el tablero para pintarse: `list_projects`
/// sigue disponible para quien no necesite etiquetas.
#[tauri::command]
pub async fn list_projects_with_tags(
    db: State<'_, Arc<Db>>,
) -> Result<Vec<ProjectWithTags>, String> {
    let projects = db.with_conn(repo::list_all_with_tags)?;
    tracing::debug!(count = projects.len(), "list_projects_with_tags");
    Ok(projects)
}

/// Devuelve un proyecto concreto.
#[tauri::command]
pub async fn get_project(db: State<'_, Arc<Db>>, id: i64) -> Result<Project, String> {
    Ok(db.with_conn(|conn| repo::get_by_id(conn, id))?)
}

/// Fija o quita la marca de favorito de un proyecto.
#[tauri::command]
pub async fn set_project_pinned(
    db: State<'_, Arc<Db>>,
    id: i64,
    pinned: bool,
) -> Result<(), String> {
    db.with_conn(|conn| repo::set_pinned(conn, id, pinned))?;
    Ok(())
}

/// Guarda las notas de un proyecto y devuelve lo que ha quedado guardado.
///
/// Un texto vacío o de solo espacios se guarda como `NULL`, y el mismo texto
/// que ya había no se vuelve a escribir. El frontend usa la respuesta para
/// quedarse con el valor real, no con el que envió.
#[tauri::command]
pub async fn set_project_notes(
    db: State<'_, Arc<Db>>,
    id: i64,
    notes: String,
) -> Result<Option<String>, String> {
    let normalized = normalize_notes(&notes)?;
    let wrote = db.with_conn(|conn| repo::set_notes(conn, id, normalized.as_deref(), now_ts()))?;
    tracing::debug!(id, wrote, "set_project_notes");
    Ok(normalized)
}

/// Lee y renderiza el README de un proyecto.
///
/// Devuelve `null` si no tiene README, o si la carpeta ya no está en disco:
/// los dos son casos normales que la interfaz explica con un mensaje.
#[tauri::command]
pub async fn get_project_readme(
    db: State<'_, Arc<Db>>,
    project_id: i64,
) -> Result<Option<ReadmePreview>, String> {
    let project = db.with_conn(|conn| repo::get_by_id(conn, project_id))?;

    tauri::async_runtime::spawn_blocking(move || readme::read(&PathBuf::from(&project.path)))
        .await
        .map_err(|err| AppError::Internal(format!("la lectura del README falló: {err}")))?
        .map_err(String::from)
}
