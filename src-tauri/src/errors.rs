//! Tipo de error global de Mosaic.
//!
//! La capa `core`/`db` trabaja siempre con [`AppError`]. Los comandos Tauri lo
//! convierten a `String` en la frontera con el frontend, tal y como exige la
//! convención del proyecto.

use std::path::Path;

/// Error de cualquier operación del núcleo de Mosaic.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Fallo de SQLite.
    #[error("error de base de datos: {0}")]
    Db(#[from] rusqlite::Error),

    /// Fallo aplicando migraciones.
    #[error("error aplicando migraciones: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    /// Fallo de entrada/salida en el sistema de ficheros.
    #[error("error de E/S: {0}")]
    Io(#[from] std::io::Error),

    /// La ruta no existe, no es un directorio o no es representable en UTF-8.
    #[error("ruta inválida: {0}")]
    InvalidPath(String),

    /// El recurso solicitado no existe en la base de datos.
    #[error("no encontrado: {0}")]
    NotFound(String),

    /// Invariante interna rota (por ejemplo, un mutex envenenado).
    #[error("error interno: {0}")]
    Internal(String),
}

impl AppError {
    /// Construye un [`AppError::InvalidPath`] a partir de una ruta y un motivo.
    pub fn invalid_path(path: &Path, reason: &str) -> Self {
        Self::InvalidPath(format!("{}: {reason}", path.display()))
    }
}

/// Convierte el error al `String` que espera el puente de comandos Tauri.
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

/// Alias de conveniencia para los resultados del núcleo.
pub type Result<T> = std::result::Result<T, AppError>;
