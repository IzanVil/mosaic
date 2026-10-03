//! Núcleo de Mosaic: organizador visual local-first de proyectos de código.
//!
//! El binario (`main.rs`) delega aquí para que la lógica sea reutilizable
//! desde los tests de integración y desde los distintos targets de Tauri.

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

/// Evento que recibe el frontend cuando termina un refresco automático del
/// estado Git. Su carga útil es un [`core::git::GitRefreshSummary`].
pub const EVENT_GIT_STATUS_REFRESHED: &str = "git-status-refreshed";
/// Evento que recibe el frontend cuando el escaneo al arrancar encontró algo.
pub const EVENT_PROJECTS_RESCANNED: &str = "projects-rescanned";

/// Espera antes del primer refresco automático, para no competir con el arranque
/// de la ventana pero dejar los indicadores al día cuanto antes.
const FIRST_REFRESH_DELAY: Duration = Duration::from_secs(5);
/// Espera antes del escaneo inicial, cuando el ajuste lo pide.
///
/// Va después del primer refresco de Git para que el tablero se pinte con la
/// caché de la sesión anterior antes de que el disco empiece a trabajar.
const STARTUP_SCAN_DELAY: Duration = Duration::from_secs(8);

/// Inicializa el logging a stdout.
///
/// El nivel se controla con la variable de entorno `MOSAIC_LOG`
/// (sintaxis de `tracing_subscriber::EnvFilter`); por defecto `info`.
fn init_tracing() {
    let filter = EnvFilter::try_from_env("MOSAIC_LOG").unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(true)
        .init();
}

/// Lanza la tarea que refresca el estado Git en segundo plano.
///
/// Hace un primer refresco poco después del arranque, porque la caché que quedó
/// de la sesión anterior puede estar muy desactualizada, y a partir de ahí
/// respeta el intervalo configurado. El intervalo se relee en cada vuelta, de
/// modo que un cambio en los ajustes surte efecto sin reiniciar; el valor `0`
/// desactiva el refresco automático sin detener la tarea.
/// Lanza el escaneo inicial si el ajuste `scan.on_startup` está activado.
///
/// Espera [`STARTUP_SCAN_DELAY`] antes de tocar el disco: al arrancar hay ya
/// bastante trabajo entre abrir la base de datos, montar la ventana y pintar el
/// tablero con lo que había de la sesión anterior, y el recorrido puede durar
/// cientos de milisegundos.
///
/// Tras escanear refresca el estado Git, porque los proyectos recién
/// descubiertos no tienen nada en la caché y sus tarjetas saldrían sin rama
/// hasta el siguiente ciclo automático.
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
                // Desactivado: se sigue consultando por si el usuario lo reactiva.
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

/// Punto de entrada compartido: configura y arranca la aplicación Tauri.
///
/// # Panics
///
/// Si la base de datos no se puede abrir o migrar, o si Tauri no consigue
/// construir la ventana principal: en ambos casos la aplicación no puede
/// continuar de ninguna forma útil.
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
            spawn_startup_scan_task(app.handle().clone(), Arc::clone(&db));
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
            commands::projects::set_project_notes,
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
