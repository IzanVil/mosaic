use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("error de base de datos: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("error aplicando migraciones: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    #[error("error de E/S: {0}")]
    Io(#[from] std::io::Error),

    #[error("error de Git: {0}")]
    Git(#[from] git2::Error),

    #[error("ruta inválida: {0}")]
    InvalidPath(String),

    #[error("no encontrado: {0}")]
    NotFound(String),

    #[error("{0}")]
    Validation(String),

    #[error("error interno: {0}")]
    Internal(String),
}

impl AppError {
    pub fn invalid_path(path: &Path, reason: &str) -> Self {
        Self::InvalidPath(format!("{}: {reason}", path.display()))
    }
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
