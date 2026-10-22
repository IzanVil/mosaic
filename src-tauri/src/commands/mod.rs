//! Comandos Tauri: la capa fina entre el frontend y `core`/`db`.
//!
//! Sin lógica de negocio: validan la entrada, llevan el trabajo bloqueante fuera
//! del hilo de la interfaz y convierten `AppError` en `CommandError`.

pub mod backup;
pub mod git;
pub mod projects;
pub mod scanner;
pub mod settings;
pub mod system;
pub mod tags;

use crate::errors::{AppError, CommandError};

/// Ejecuta `f` fuera del hilo de la interfaz; si la tarea se cae, el error dice qué hacía (`what`).
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
