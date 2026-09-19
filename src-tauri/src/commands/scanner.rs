use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use crate::core::scanner::{run_full_scan, ScanSummary};
use crate::db::repositories::scan_paths as repo;
use crate::db::Db;
use crate::errors::AppError;

fn canonical_dir(raw: &str) -> Result<PathBuf, AppError> {
    let path = PathBuf::from(raw);
    let canonical = crate::core::strip_verbatim_prefix(
        std::fs::canonicalize(&path)
            .map_err(|err| AppError::invalid_path(&path, &err.to_string()))?,
    );

    if !canonical.is_dir() {
        return Err(AppError::invalid_path(&canonical, "no es un directorio"));
    }
    Ok(canonical)
}

#[tauri::command]
pub async fn add_scan_path(db: State<'_, Arc<Db>>, path: String) -> Result<repo::ScanPath, String> {
    let canonical = canonical_dir(&path)?;
    let path_str = canonical
        .to_str()
        .ok_or_else(|| AppError::invalid_path(&canonical, "no es UTF-8 válido"))?
        .to_string();

    let added = db.with_conn(|conn| repo::add(conn, &path_str, crate::core::now_ts()))?;
    tracing::info!(path = %path_str, "ruta de escaneo añadida");
    Ok(added)
}

#[tauri::command]
pub async fn list_scan_paths(db: State<'_, Arc<Db>>) -> Result<Vec<repo::ScanPath>, String> {
    let paths = db.with_conn(repo::list)?;
    tracing::debug!(count = paths.len(), "list_scan_paths");
    Ok(paths)
}

#[tauri::command]
pub async fn remove_scan_path(db: State<'_, Arc<Db>>, id: i64) -> Result<bool, String> {
    let removed = db.with_conn(|conn| repo::delete(conn, id))?;
    tracing::info!(id, removed, "ruta de escaneo eliminada");
    Ok(removed)
}

#[tauri::command]
pub async fn set_scan_path_enabled(
    db: State<'_, Arc<Db>>,
    id: i64,
    enabled: bool,
) -> Result<(), String> {
    db.with_conn(|conn| repo::set_enabled(conn, id, enabled))?;
    Ok(())
}

#[tauri::command]
pub async fn scan_all_paths(db: State<'_, Arc<Db>>) -> Result<ScanSummary, String> {
    let db = Arc::clone(&db);

    tauri::async_runtime::spawn_blocking(move || run_full_scan(&db))
        .await
        .map_err(|err| AppError::Internal(format!("la tarea de escaneo falló: {err}")))?
        .map_err(String::from)
}
