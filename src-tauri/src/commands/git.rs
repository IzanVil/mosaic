use std::sync::Arc;

use tauri::State;

use crate::core::git::{refresh_all, refresh_one, GitRefreshSummary};
use crate::db::repositories::git_status::{self as repo, GitStatusEntry};
use crate::db::Db;
use crate::errors::AppError;

#[tauri::command]
pub async fn list_git_status(db: State<'_, Arc<Db>>) -> Result<Vec<GitStatusEntry>, String> {
    let entries = db.with_conn(repo::list_all)?;
    tracing::debug!(count = entries.len(), "list_git_status");
    Ok(entries)
}

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

#[tauri::command]
pub async fn refresh_all_git_status(db: State<'_, Arc<Db>>) -> Result<GitRefreshSummary, String> {
    let db = Arc::clone(&db);

    tauri::async_runtime::spawn_blocking(move || refresh_all(&db))
        .await
        .map_err(|err| AppError::Internal(format!("la tarea de refresco falló: {err}")))?
        .map_err(String::from)
}
