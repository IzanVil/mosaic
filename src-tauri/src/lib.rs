//! Núcleo de Mosaic: organizador visual local-first de proyectos de código.
//!
//! El binario (`main.rs`) delega aquí para que la lógica sea reutilizable
//! desde tests de integración y desde los distintos targets de Tauri.

use tracing_subscriber::{fmt, EnvFilter};

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
/// Solo si Tauri no consigue construir la ventana principal, en cuyo caso la
/// aplicación no puede continuar de ninguna forma útil.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();
    tracing::info!("iniciando Mosaic v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error al arrancar la aplicación Tauri");
}
