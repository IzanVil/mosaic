//! Comandos Tauri: capa fina de adaptación entre el frontend y `core`/`db`.
//!
//! Aquí no vive lógica de negocio: solo validación de entrada, conversión de
//! errores a `String` y traslado del trabajo bloqueante fuera del hilo de la UI.

pub mod backup;
pub mod git;
pub mod projects;
pub mod scanner;
pub mod settings;
pub mod system;
pub mod tags;

use crate::errors::{AppError, CommandError};

/// Ejecuta `f` fuera del hilo de la interfaz. Si la tarea se cae, el error dice
/// qué se estaba haciendo (`what`, por ejemplo «la lectura del historial»).
pub(crate) async fn blocking<T, F>(what: &str, f: F) -> Result<T, CommandError>
where
    F: FnOnce() -> crate::errors::Result<T> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|err| {
            AppError::internal("task_failed", format!("{what} falló: {err}"))
                .with("detail", err.to_string())
        })?
        .map_err(CommandError::from)
}
