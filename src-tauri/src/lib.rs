pub mod commands;
pub mod config;
pub mod core;
pub mod db;
pub mod errors;

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tracing_subscriber::{fmt, EnvFilter};

use crate::db::Db;

pub const EVENT_GIT_STATUS_REFRESHED: &str = "git-status-refreshed";

const FIRST_REFRESH_DELAY: Duration = Duration::from_secs(5);

fn init_tracing() {
    let filter = EnvFilter::try_from_env("MOSAIC_LOG").unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(true)
        .init();
}

fn spawn_git_refresh_task(app: AppHandle, db: Arc<Db>) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_REFRESH_DELAY).await;

        loop {
            let minutes = match db.with_conn(config::settings::load) {
                Ok(settings) => settings.git_refresh_interval_minutes,
                Err(err) => {
                    tracing::warn!(%err, "no se pudieron leer los ajustes de refresco");
                    config::settings::DEFAULT_GIT_REFRESH_MINUTES
                }
            };

            if minutes == 0 {
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }

            let task_db = Arc::clone(&db);
            let refreshed =
                tauri::async_runtime::spawn_blocking(move || core::git::refresh_all(&task_db))
                    .await;

            match refreshed {
                Ok(Ok(summary)) => {
                    if let Err(err) = app.emit(EVENT_GIT_STATUS_REFRESHED, summary) {
                        tracing::warn!(%err, "no se pudo notificar el refresco al frontend");
                    }
                }
                Ok(Err(err)) => tracing::warn!(%err, "el refresco automático falló"),
                Err(err) => tracing::warn!(%err, "la tarea de refresco se interrumpió"),
            }

            tokio::time::sleep(Duration::from_secs(minutes * 60)).await;
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();
    tracing::info!("iniciando Mosaic v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let path = db::default_db_path()?;
            let db = Arc::new(Db::open(&path)?);
            app.manage(Arc::clone(&db));
            spawn_git_refresh_task(app.handle().clone(), db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::projects::list_projects,
            commands::projects::get_project,
            commands::scanner::add_scan_path,
            commands::scanner::list_scan_paths,
            commands::scanner::remove_scan_path,
            commands::scanner::set_scan_path_enabled,
            commands::scanner::scan_all_paths,
            commands::projects::set_project_pinned,
            commands::system::list_detected_apps,
            commands::system::open_in,
            commands::system::get_preferred_apps,
            commands::system::set_preferred_app,
            commands::git::list_git_status,
            commands::git::refresh_git_status,
            commands::git::refresh_all_git_status,
            commands::projects::list_projects_with_tags,
            commands::tags::create_tag,
            commands::tags::list_tags,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::assign_tag,
            commands::tags::unassign_tag,
            commands::tags::list_tags_for_project,
            commands::settings::get_view_state,
            commands::settings::set_view_state,
        ])
        .run(tauri::generate_context!())
        .expect("error al arrancar la aplicación Tauri");
}
