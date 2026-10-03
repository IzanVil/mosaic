//! Comandos de lectura y refresco del estado Git.

use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use crate::core::git::{
    read_branches, read_history, refresh_all, refresh_one, Branches, CommitInfo, GitRefreshSummary,
    HISTORY_LIMIT,
};
use crate::db::repositories::git_status::{self as repo, GitStatusEntry};
use crate::db::repositories::projects as projects_repo;
use crate::db::Db;
use crate::errors::AppError;

/// Devuelve toda la caché de estado Git, para que el frontend la indexe por proyecto.
#[tauri::command]
pub async fn list_git_status(db: State<'_, Arc<Db>>) -> Result<Vec<GitStatusEntry>, String> {
    let entries = db.with_conn(repo::list_all)?;
    tracing::debug!(count = entries.len(), "list_git_status");
    Ok(entries)
}

/// Relee el estado Git de un proyecto concreto.
///
/// Devuelve `null` si el proyecto no es un repositorio Git.
#[tauri::command]
pub async fn refresh_git_status(
    db: State<'_, Arc<Db>>,
    project_id: i64,
) -> Result<Option<GitStatusEntry>, String> {
    let db = Arc::clone(&db);

    tauri::async_runtime::spawn_blocking(move || refresh_one(&db, project_id))
        .await
        .map_err(|err| AppError::Internal(format!("la tarea de refresco falló: {err}")))?
        .map_err(String::from)
}

/// Relee el estado Git de todos los proyectos que son repositorios.
///
/// La lectura de disco es bloqueante, así que se ejecuta fuera del hilo de la
/// interfaz.
#[tauri::command]
pub async fn refresh_all_git_status(db: State<'_, Arc<Db>>) -> Result<GitRefreshSummary, String> {
    let db = Arc::clone(&db);

    tauri::async_runtime::spawn_blocking(move || refresh_all(&db))
        .await
        .map_err(|err| AppError::Internal(format!("la tarea de refresco falló: {err}")))?
        .map_err(String::from)
}

/// Ruta del proyecto si tiene sentido leer su repositorio: está en disco y es
/// un repositorio Git. Si no, `None`, y el comando responde con una lista vacía
/// en lugar de un error de Git que la interfaz no sabría explicar.
fn readable_repo(db: &Db, project_id: i64) -> Result<Option<PathBuf>, AppError> {
    let project = db.with_conn(|conn| projects_repo::get_by_id(conn, project_id))?;
    Ok((project.is_git_repo && !project.missing).then(|| PathBuf::from(project.path)))
}

/// Devuelve los últimos commits del proyecto, del más reciente al más antiguo.
#[tauri::command]
pub async fn get_project_history(
    db: State<'_, Arc<Db>>,
    project_id: i64,
) -> Result<Vec<CommitInfo>, String> {
    let Some(path) = readable_repo(&db, project_id)? else {
        return Ok(Vec::new());
    };

    tauri::async_runtime::spawn_blocking(move || read_history(&path, HISTORY_LIMIT))
        .await
        .map_err(|err| AppError::Internal(format!("la lectura del historial falló: {err}")))?
        .map_err(String::from)
}

/// Devuelve las ramas locales y las remotas conocidas del proyecto.
#[tauri::command]
pub async fn get_project_branches(
    db: State<'_, Arc<Db>>,
    project_id: i64,
) -> Result<Branches, String> {
    let Some(path) = readable_repo(&db, project_id)? else {
        return Ok(Branches::default());
    };

    tauri::async_runtime::spawn_blocking(move || read_branches(&path))
        .await
        .map_err(|err| AppError::Internal(format!("la lectura de las ramas falló: {err}")))?
        .map_err(String::from)
}
