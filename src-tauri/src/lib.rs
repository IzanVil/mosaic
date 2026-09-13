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

use tauri::Manager;
use tracing_subscriber::{fmt, EnvFilter};

use crate::db::Db;

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
            let db = Db::open(&path)?;
            app.manage(Arc::new(db));
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
        ])
        .run(tauri::generate_context!())
        .expect("error al arrancar la aplicación Tauri");
}
