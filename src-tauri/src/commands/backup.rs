//! Comandos de copia de seguridad.

use std::path::Path;
use std::sync::Arc;

use tauri::State;

use crate::core::backup::{self, Backup, ExportSummary, ImportPlan, MAX_BACKUP_BYTES};
use crate::db::Db;
use crate::errors::{AppError, CommandError};
use crate::GitRefreshSignal;

/// El fichero no se puede leer: no existe, es una carpeta, faltan permisos…
fn unreadable(ruta: &Path, err: &std::io::Error) -> AppError {
    AppError::invalid_path(ruta, "path_unreadable", &err.to_string())
        .with("detail", err.to_string())
}

/// Lee y valida la copia de `path`. Un fichero demasiado grande o que no es
/// texto UTF-8 no es una copia de Mosaic; cualquier otro fallo de lectura es
/// un problema de la ruta.
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn ruta(dir: &tempfile::TempDir, nombre: &str) -> String {
        dir.path().join(nombre).to_string_lossy().into_owned()
    }

    fn copia() -> Backup {
        let db = Db::open_in_memory().unwrap();
        db.with_conn(|conn| backup::build_backup(conn, true, 1_700_000_000, "0.0.0"))
            .unwrap()
    }

    #[test]
    fn reads_a_backup_written_by_mosaic() {
        let dir = tempfile::tempdir().unwrap();
        let path = ruta(&dir, "copia.json");
        let escrita = copia();
        std::fs::write(&path, backup::to_json(&escrita).unwrap()).unwrap();

        assert_eq!(read_backup(&path).unwrap(), escrita);
    }

    #[test]
    fn a_valid_backup_over_the_size_limit_is_rejected() {
        // Rellena con espacios, que el JSON admite: sin el tope, se leería bien.
        let dir = tempfile::tempdir().unwrap();
        let path = ruta(&dir, "enorme.json");
        let json = backup::to_json(&copia()).unwrap();
        let relleno = MAX_BACKUP_BYTES as usize + 1 - json.len();
        let mut fichero = std::fs::File::create(&path).unwrap();
        fichero.write_all(json.as_bytes()).unwrap();
        fichero.write_all(&vec![b' '; relleno]).unwrap();
        drop(fichero);

        assert_eq!(read_backup(&path).unwrap_err().code(), "backup_not_mosaic");
    }

    #[test]
    fn a_file_that_is_not_utf8_is_not_a_backup_rather_than_unreadable() {
        let dir = tempfile::tempdir().unwrap();
        let path = ruta(&dir, "imagen.png");
        std::fs::write(&path, [0x89, b'P', b'N', b'G', 0xff, 0xfe]).unwrap();

        assert_eq!(read_backup(&path).unwrap_err().code(), "backup_not_mosaic");
    }

    #[test]
    fn a_missing_file_is_unreadable_and_carries_its_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = ruta(&dir, "no-existe.json");

        let err = CommandError::from(read_backup(&path).unwrap_err());
        assert_eq!(err.code, "path_unreadable");
        assert_eq!(err.params["path"], path);
        assert!(err.params["detail"].is_string());
    }

    #[test]
    fn a_folder_is_unreadable_not_a_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_string_lossy().into_owned();

        assert_eq!(read_backup(&path).unwrap_err().code(), "path_unreadable");
    }
}
