//! Comandos de integración con el sistema: abrir proyectos en aplicaciones
//! externas y descubrir cuáles hay instaladas.

use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use crate::config::settings;
use crate::core::launcher::{detect, open_external as open_url, open_path, AppKind, DetectedApp};
use crate::db::repositories::projects as projects_repo;
use crate::db::Db;
use crate::errors::AppError;

/// Devuelve los IDEs y terminales reconocidos que hay en el sistema.
#[tauri::command]
pub async fn list_detected_apps() -> Result<Vec<DetectedApp>, String> {
    let mut apps = detect(AppKind::Ide);
    apps.extend(detect(AppKind::Terminal));

    tracing::debug!(count = apps.len(), "list_detected_apps");
    Ok(apps)
}

/// Abre un proyecto en el IDE, la terminal o el explorador de archivos.
///
/// Registra el momento de apertura en `last_opened_at`, pero solo si el
/// lanzamiento ha tenido éxito.
#[tauri::command]
pub async fn open_in(db: State<'_, Arc<Db>>, kind: AppKind, project_id: i64) -> Result<(), String> {
    let db = Arc::clone(&db);

    tauri::async_runtime::spawn_blocking(move || {
        let (project, config) = db.with_conn(|conn| {
            Ok((
                projects_repo::get_by_id(conn, project_id)?,
                settings::load(conn)?,
            ))
        })?;

        let preferred = match kind {
            AppKind::Ide => config.preferred_ide,
            AppKind::Terminal => config.preferred_terminal,
            AppKind::FileManager => String::new(),
        };

        open_path(kind, &PathBuf::from(&project.path), Some(&preferred))?;
        db.with_conn(|conn| {
            projects_repo::touch_last_opened(conn, project_id, crate::core::now_ts())
        })
    })
    .await
    .map_err(|err| AppError::Internal(format!("la tarea de apertura falló: {err}")))?
    .map_err(String::from)
}

/// Aplicaciones preferidas actualmente guardadas.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PreferredApps {
    /// Ejecutable del IDE preferido; vacío significa "el primero disponible".
    pub ide: String,
    /// Ejecutable del terminal preferido; vacío significa "el primero disponible".
    pub terminal: String,
}

/// Devuelve las aplicaciones preferidas, para poder marcarlas en los ajustes.
#[tauri::command]
pub async fn get_preferred_apps(db: State<'_, Arc<Db>>) -> Result<PreferredApps, String> {
    let config = db.with_conn(settings::load)?;
    Ok(PreferredApps {
        ide: config.preferred_ide,
        terminal: config.preferred_terminal,
    })
}

/// Guarda el IDE o el terminal preferido. Una cadena vacía vuelve a "el primero
/// que se detecte".
#[tauri::command]
pub async fn set_preferred_app(
    db: State<'_, Arc<Db>>,
    kind: AppKind,
    app_id: String,
) -> Result<(), String> {
    let key = match kind {
        AppKind::Ide => settings::KEY_PREFERRED_IDE,
        AppKind::Terminal => settings::KEY_PREFERRED_TERMINAL,
        AppKind::FileManager => {
            return Err(AppError::InvalidPath(
                "el explorador de archivos lo elige el sistema, no Mosaic".into(),
            )
            .into());
        }
    };

    db.with_conn(|conn| settings::set_raw(conn, key, &app_id))?;
    tracing::info!(?kind, app = %app_id, "aplicación preferida guardada");
    Ok(())
}

/// Abre un enlace web en el navegador predeterminado.
///
/// Lo usan los enlaces del README. Solo pasan `http` y `https`: la lista
/// blanca y su porqué viven en [`crate::core::launcher::validate_external_url`].
#[tauri::command]
pub async fn open_external(url: String) -> Result<(), String> {
    open_url(&url)?;
    tracing::debug!(%url, "open_external");
    Ok(())
}
