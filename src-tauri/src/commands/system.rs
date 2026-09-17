use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use crate::config::settings;
use crate::core::launcher::{detect, open_path, AppKind, DetectedApp};
use crate::db::repositories::projects as projects_repo;
use crate::db::Db;
use crate::errors::AppError;

#[tauri::command]
pub async fn list_detected_apps() -> Result<Vec<DetectedApp>, String> {
    let mut apps = detect(AppKind::Ide);
    apps.extend(detect(AppKind::Terminal));

    tracing::debug!(count = apps.len(), "list_detected_apps");
    Ok(apps)
}

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PreferredApps {
    pub ide: String,
    pub terminal: String,
}

#[tauri::command]
pub async fn get_preferred_apps(db: State<'_, Arc<Db>>) -> Result<PreferredApps, String> {
    let config = db.with_conn(settings::load)?;
    Ok(PreferredApps {
        ide: config.preferred_ide,
        terminal: config.preferred_terminal,
    })
}

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
