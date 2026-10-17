//! Comandos de configuración de rutas y lanzamiento de escaneos.

use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use crate::core::scanner::{run_full_scan, ScanSummary};
use crate::db::repositories::scan_paths as repo;
use crate::db::Db;
use crate::errors::{AppError, CommandError};

use super::blocking;

/// Normaliza una ruta recibida del frontend: debe existir y ser un directorio.
fn canonical_dir(raw: &str) -> Result<PathBuf, AppError> {
    let path = PathBuf::from(raw);
    let canonical =
        crate::core::strip_verbatim_prefix(std::fs::canonicalize(&path).map_err(|err| {
            AppError::invalid_path(&path, "path_unreadable", &err.to_string())
                .with("detail", err.to_string())
        })?);

    if !canonical.is_dir() {
        return Err(AppError::invalid_path(
            &canonical,
            "path_not_dir",
            "no es un directorio",
        ));
    }
    Ok(canonical)
}

/// Registra una ruta raíz de escaneo.
#[tauri::command]
pub async fn add_scan_path(
    db: State<'_, Arc<Db>>,
    path: String,
) -> Result<repo::ScanPath, CommandError> {
    let canonical = canonical_dir(&path)?;
    let path_str = canonical
        .to_str()
        .ok_or_else(|| AppError::invalid_path(&canonical, "path_not_utf8", "no es UTF-8 válido"))?
        .to_string();

    let added = db.with_conn(|conn| repo::add(conn, &path_str, crate::core::now_ts()))?;
    tracing::info!(path = %path_str, "ruta de escaneo añadida");
    Ok(added)
}

/// Devuelve todas las rutas raíz configuradas.
#[tauri::command]
pub async fn list_scan_paths(db: State<'_, Arc<Db>>) -> Result<Vec<repo::ScanPath>, CommandError> {
    let paths = db.with_conn(repo::list)?;
    tracing::debug!(count = paths.len(), "list_scan_paths");
    Ok(paths)
}

/// Elimina una ruta raíz. Los proyectos ya descubiertos se conservan.
#[tauri::command]
pub async fn remove_scan_path(db: State<'_, Arc<Db>>, id: i64) -> Result<bool, CommandError> {
    let removed = db.with_conn(|conn| repo::delete(conn, id))?;
    tracing::info!(id, removed, "ruta de escaneo eliminada");
    Ok(removed)
}

/// Habilita o deshabilita una ruta raíz sin borrarla.
#[tauri::command]
pub async fn set_scan_path_enabled(
    db: State<'_, Arc<Db>>,
    id: i64,
    enabled: bool,
) -> Result<(), CommandError> {
    db.with_conn(|conn| repo::set_enabled(conn, id, enabled))?;
    Ok(())
}

/// Escanea todas las rutas habilitadas y sincroniza la lista de proyectos.
///
/// El recorrido del sistema de ficheros es bloqueante, así que se ejecuta en el
/// pool de tareas bloqueantes para no congelar la interfaz.
#[tauri::command]
pub async fn scan_all_paths(db: State<'_, Arc<Db>>) -> Result<ScanSummary, CommandError> {
    let db = Arc::clone(&db);

    blocking("la tarea de escaneo", move || run_full_scan(&db)).await
}
