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

#[cfg(test)]
mod tests {
    use std::path::{Component, Path};

    use super::*;

    #[test]
    fn resolves_a_roundabout_path_to_the_real_folder() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("a").join("b")).unwrap();
        let rodeo = dir.path().join("a").join("..").join("a").join("b");

        let canonica = canonical_dir(rodeo.to_str().unwrap()).unwrap();
        assert!(canonica.is_absolute());
        assert!(canonica.ends_with(Path::new("a").join("b")));
        assert!(!canonica.components().any(|c| c == Component::ParentDir));
        // En Windows, `canonicalize` devuelve `\\?\C:\…`: no debe llegar a la base.
        assert!(
            !canonica.to_string_lossy().starts_with(r"\\?\"),
            "{canonica:?}"
        );
    }

    #[test]
    fn a_path_that_does_not_exist_is_unreadable() {
        let dir = tempfile::tempdir().unwrap();
        let falta = dir.path().join("no-existe");

        let err = canonical_dir(falta.to_str().unwrap()).unwrap_err();
        assert_eq!(err.code(), "path_unreadable");
    }

    #[test]
    fn a_file_is_not_a_scan_folder() {
        let dir = tempfile::tempdir().unwrap();
        let fichero = dir.path().join("notas.txt");
        std::fs::write(&fichero, "hola").unwrap();

        let err = CommandError::from(canonical_dir(fichero.to_str().unwrap()).unwrap_err());
        assert_eq!(err.code, "path_not_dir");
        assert!(err.params["path"].as_str().unwrap().ends_with("notas.txt"));
    }
}
