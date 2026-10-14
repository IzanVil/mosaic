//! Comandos de copia de seguridad.

use std::path::Path;
use std::sync::Arc;

use tauri::State;

use crate::core::backup::{self, Backup, ExportSummary, ImportPlan, MAX_BACKUP_BYTES};
use crate::db::Db;
use crate::errors::{AppError, CommandError};
use crate::GitRefreshSignal;

fn unreadable(ruta: &Path, err: &std::io::Error) -> AppError {
    AppError::invalid_path(ruta, "path_unreadable", &err.to_string())
        .with("detail", err.to_string())
}

fn read_backup(path: &str) -> Result<Backup, AppError> {
    let ruta = Path::new(path);
    let size = std::fs::metadata(ruta)
        .map_err(|err| unreadable(ruta, &err))?
        .len();
    if size > MAX_BACKUP_BYTES {
        return Err(backup::not_a_backup());
    }
    let text = std::fs::read_to_string(ruta).map_err(|err| match err.kind() {
        std::io::ErrorKind::InvalidData => backup::not_a_backup(),
        _ => unreadable(ruta, &err),
    })?;
    backup::parse_backup(&text)
}

/// Escribe la copia en `path`.
#[tauri::command]
pub async fn export_backup(
    db: State<'_, Arc<Db>>,
    path: String,
    include_work: bool,
) -> Result<ExportSummary, CommandError> {
    let copia = db.with_conn(|conn| {
        backup::build_backup(
            conn,
            include_work,
            crate::core::now_ts(),
            env!("CARGO_PKG_VERSION"),
        )
    })?;
    std::fs::write(&path, backup::to_json(&copia)?).map_err(AppError::from)?;

    let summary = ExportSummary {
        scan_paths: copia.scan_paths.len(),
        tags: copia.work.as_ref().map_or(0, |w| w.tags.len()),
        projects: copia.work.as_ref().map_or(0, |w| w.projects.len()),
    };
    tracing::info!(%path, ?summary, "copia exportada");
    Ok(summary)
}

/// Lee una copia y cuenta lo que haría importarla, sin escribir nada.
#[tauri::command]
pub async fn preview_import(
    db: State<'_, Arc<Db>>,
    path: String,
) -> Result<ImportPlan, CommandError> {
    let copia = read_backup(&path)?;
    Ok(db.with_conn(|conn| backup::plan_import(conn, &copia))?)
}

/// Importa una copia.
#[tauri::command]
pub async fn apply_import(
    db: State<'_, Arc<Db>>,
    signal: State<'_, Arc<GitRefreshSignal>>,
    path: String,
) -> Result<ImportPlan, CommandError> {
    let copia = read_backup(&path)?;
    let plan =
        db.with_conn_mut(|conn| backup::apply_import(conn, &copia, crate::core::now_ts()))?;
    if plan.git_interval_changed {
        signal.0.notify_one();
    }
    tracing::info!(%path, ?plan, "copia importada");
    Ok(plan)
}
