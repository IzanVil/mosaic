//! Núcleo de Mosaic. `main.rs` delega aquí para que los tests de integración lo usen.

pub mod commands;
pub mod config;
pub mod core;
pub mod db;
pub mod errors;
pub mod model;

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;

use tauri::{AppHandle, Emitter, Manager};
use tracing_subscriber::{fmt, EnvFilter};

use crate::db::Db;

/// Evento del refresco automático de Git, con un [`core::git::GitRefreshSummary`] de lo que cambió.
pub const EVENT_GIT_STATUS_REFRESHED: &str = "git-status-refreshed";
/// Evento que recibe el frontend cuando el escaneo al arrancar encontró algo.
pub const EVENT_PROJECTS_RESCANNED: &str = "projects-rescanned";

/// Espera antes del primer refresco: sin estorbar al arranque, pero pronto, que la caché envejece.
const FIRST_REFRESH_DELAY: Duration = Duration::from_secs(5);
/// Espera antes del escaneo inicial: después del primer refresco, con el tablero ya pintado.
const STARTUP_SCAN_DELAY: Duration = Duration::from_secs(8);

/// Logging a stdout; el nivel sale de `MOSAIC_LOG` (sintaxis de `EnvFilter`), `info` por defecto.
fn init_tracing() {
    let filter = EnvFilter::try_from_env("MOSAIC_LOG").unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(true)
        .init();
}

/// Lanza el escaneo inicial si el ajuste `scan.on_startup` lo pide.
fn spawn_startup_scan_task(app: AppHandle, db: Arc<Db>) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(STARTUP_SCAN_DELAY).await;

        let scan_db = Arc::clone(&db);
        let scanned =
            tauri::async_runtime::spawn_blocking(move || core::scanner::run_startup_scan(&scan_db))
                .await;

        let summary = match scanned {
            Ok(Ok(Some(summary))) => summary,
            Ok(Ok(None)) => return,
            Ok(Err(err)) => {
                tracing::warn!(%err, "el escaneo al arrancar falló");
                return;
            }
            Err(err) => {
                tracing::warn!(%err, "la tarea de escaneo al arrancar se interrumpió");
                return;
            }
        };

        tracing::info!(?summary, "escaneo al arrancar terminado");

        // Los proyectos nuevos no tienen caché: sin esto saldrían sin rama hasta el siguiente ciclo.
        let git_db = Arc::clone(&db);
        match tauri::async_runtime::spawn_blocking(move || core::git::refresh_all(&git_db)).await {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => tracing::warn!(%err, "el refresco posterior al escaneo falló"),
            Err(err) => tracing::warn!(%err, "la tarea de refresco se interrumpió"),
        }

        if let Err(err) = app.emit(EVENT_PROJECTS_RESCANNED, summary) {
            tracing::warn!(%err, "no se pudo notificar el escaneo al frontend");
        }
    });
}

/// Despierta el refresco de Git al cambiar el intervalo: si no, bajar de 60 a 1 tardaba una hora.
#[derive(Default)]
pub struct GitRefreshSignal(pub Notify);

/// Espera `duration` o la señal, lo que llegue antes; `true` si fue la señal.
async fn wait_or_wake(duration: Duration, signal: &GitRefreshSignal) -> bool {
    // Si la señal llega sin nadie esperando, `Notify` la guarda: un cambio no se pierde.
    tokio::select! {
        _ = tokio::time::sleep(duration) => false,
        _ = signal.0.notified() => true,
    }
}

/// Refresca el estado Git en segundo plano: al poco de arrancar y luego cada N minutos.
fn spawn_git_refresh_task(app: AppHandle, db: Arc<Db>, signal: Arc<GitRefreshSignal>) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_REFRESH_DELAY).await;

        loop {
            // Se relee en cada vuelta: un cambio de ajuste vale sin reiniciar.
            let minutes = match db.with_conn(db::repositories::settings::load) {
                Ok(settings) => settings.git_refresh_interval_minutes,
                Err(err) => {
                    tracing::warn!(%err, "no se pudieron leer los ajustes de refresco");
                    config::settings::DEFAULT_GIT_REFRESH_MINUTES
                }
            };

            if minutes == 0 {
                // Desactivado: se sigue consultando por si el usuario lo
                // reactiva, y la señal lo despierta en cuanto lo hace.
                wait_or_wake(Duration::from_secs(60), &signal).await;
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

            if wait_or_wake(Duration::from_secs(minutes * 60), &signal).await {
                tracing::info!(
                    "intervalo de Git cambiado: se relee ya y se empieza a contar de nuevo"
                );
            }
        }
    });
}

/// Configura y arranca la aplicación Tauri.
///
/// # Panics
///
/// Si la base de datos no se abre o no migra, o si Tauri no construye la ventana.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();
    tracing::info!("iniciando Mosaic v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let path = db::default_db_path()?;
            let db = Arc::new(Db::open(&path)?);
            let signal = Arc::new(GitRefreshSignal::default());
            app.manage(Arc::clone(&db));
            app.manage(Arc::clone(&signal));
            spawn_startup_scan_task(app.handle().clone(), Arc::clone(&db));
            spawn_git_refresh_task(app.handle().clone(), db, signal);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scanner::add_scan_path,
            commands::scanner::list_scan_paths,
            commands::scanner::remove_scan_path,
            commands::scanner::set_scan_path_enabled,
            commands::scanner::scan_all_paths,
            commands::projects::set_project_pinned,
            commands::projects::set_project_notes,
            commands::projects::get_project_readme,
            commands::system::list_detected_apps,
            commands::system::open_in,
            commands::system::open_external,
            commands::system::get_preferred_apps,
            commands::system::set_preferred_app,
            commands::git::list_git_status,
            commands::git::refresh_all_git_status,
            commands::git::get_project_history,
            commands::git::get_project_branches,
            commands::projects::list_projects_with_tags,
            commands::tags::create_tag,
            commands::tags::list_tags,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::assign_tag,
            commands::tags::unassign_tag,
            commands::settings::get_view_state,
            commands::settings::set_view_state,
            commands::settings::get_advanced_settings,
            commands::settings::set_advanced_settings,
            commands::backup::export_backup,
            commands::backup::preview_import,
            commands::backup::apply_import,
        ])
        .run(tauri::generate_context!())
        .expect("error al arrancar la aplicación Tauri");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn wait_or_wake_returns_early_when_signalled() {
        let signal = GitRefreshSignal::default();
        signal.0.notify_one();
        let inicio = std::time::Instant::now();
        assert!(wait_or_wake(Duration::from_secs(30), &signal).await);
        assert!(
            inicio.elapsed() < Duration::from_secs(1),
            "no espera la media hora"
        );
    }

    #[tokio::test]
    async fn wait_or_wake_times_out_without_a_signal() {
        let signal = GitRefreshSignal::default();
        assert!(!wait_or_wake(Duration::from_millis(20), &signal).await);
    }
}
