//! Errores de Mosaic. `core` y `db` usan [`AppError`]; los comandos lo convierten
//! en [`CommandError`], con un código estable y sus datos para que el frontend
//! lo diga en el idioma de la interfaz.

use std::fmt;
use std::path::Path;

use serde::Serialize;
use serde_json::{Map, Value};

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

    /// Fallo leyendo un repositorio Git.
    #[error("error de Git: {0}")]
    Git(#[from] git2::Error),

    /// La ruta no existe, no es un directorio o no es representable en UTF-8.
    #[error("ruta inválida: {0}")]
    InvalidPath(Coded),

    /// El recurso solicitado no existe en la base de datos.
    #[error("no encontrado: {0}")]
    NotFound(Coded),

    /// La entrada del usuario no cumple una regla de negocio.
    #[error("{0}")]
    Validation(Coded),

    /// Algo que no depende del usuario falló: un mutex, una tarea de fondo, lanzar un programa.
    #[error("error interno: {0}")]
    Internal(Coded),
}

/// Mensaje con un código estable y los datos que necesita su texto.
#[derive(Debug)]
pub struct Coded {
    /// Código que el frontend busca como `error.<code>`.
    pub code: &'static str,
    /// Datos del texto: nombres, rutas, límites.
    pub params: Map<String, Value>,
    /// El mensaje en español, para los logs y por si el frontend no conoce el código.
    pub message: String,
}

impl Coded {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            params: Map::new(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Coded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl AppError {
    /// Entrada que no cumple una regla de negocio.
    pub fn validation(code: &'static str, message: impl Into<String>) -> Self {
        Self::Validation(Coded::new(code, message))
    }

    /// Recurso que no existe.
    pub fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self::NotFound(Coded::new(code, message))
    }

    /// Fallo interno.
    pub fn internal(code: &'static str, message: impl Into<String>) -> Self {
        Self::Internal(Coded::new(code, message))
    }

    /// Ruta inválida, con la ruta ya puesta entre los datos como `path`.
    pub fn invalid_path(path: &Path, code: &'static str, reason: &str) -> Self {
        let shown = path.display().to_string();
        Self::InvalidPath(Coded::new(code, format!("{shown}: {reason}"))).with("path", shown)
    }

    /// Añade un dato al texto del error. Los errores de librería no llevan datos.
    pub fn with(mut self, key: &str, value: impl Into<Value>) -> Self {
        if let Self::InvalidPath(coded)
        | Self::NotFound(coded)
        | Self::Validation(coded)
        | Self::Internal(coded) = &mut self
        {
            coded.params.insert(key.to_string(), value.into());
        }
        self
    }

    /// Añade como `cause` otro error, para que el frontend traduzca también el motivo.
    pub fn because(self, cause: AppError) -> Self {
        let cause = serde_json::to_value(CommandError::from(cause)).unwrap_or(Value::Null);
        self.with("cause", cause)
    }

    /// Código estable del error.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Db(_) => "db",
            Self::Migration(_) => "migration",
            Self::Io(_) => "io",
            Self::Git(_) => "git",
            Self::InvalidPath(coded)
            | Self::NotFound(coded)
            | Self::Validation(coded)
            | Self::Internal(coded) => coded.code,
        }
    }
}

/// Error tal y como lo recibe el frontend.
#[derive(Debug, Serialize)]
pub struct CommandError {
    /// Código estable; el frontend lo traduce como `error.<code>`.
    pub code: &'static str,
    /// Datos del texto. En los errores de librería, `detail` con su mensaje.
    pub params: Map<String, Value>,
    /// El mensaje en español, por si el frontend no conoce el código.
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(err: AppError) -> Self {
        let code = err.code();
        let message = err.to_string();
        let params = match err {
            AppError::InvalidPath(coded)
            | AppError::NotFound(coded)
            | AppError::Validation(coded)
            | AppError::Internal(coded) => coded.params,
            AppError::Db(inner) => detail(inner),
            AppError::Migration(inner) => detail(inner),
            AppError::Io(inner) => detail(inner),
            AppError::Git(inner) => detail(inner),
        };
        Self {
            code,
            params,
            message,
        }
    }
}

fn detail(inner: impl fmt::Display) -> Map<String, Value> {
    let mut params = Map::new();
    params.insert("detail".into(), Value::String(inner.to_string()));
    params
}

/// Alias de conveniencia para los resultados del núcleo.
pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_coded_error_keeps_its_message_and_carries_its_data() {
        let err =
            AppError::validation("tag_name_taken", "ya existe una etiqueta llamada «Cliente»")
                .with("name", "Cliente");
        assert_eq!(err.to_string(), "ya existe una etiqueta llamada «Cliente»");

        let sent = CommandError::from(err);
        assert_eq!(sent.code, "tag_name_taken");
        assert_eq!(sent.params["name"], "Cliente");
        assert_eq!(sent.message, "ya existe una etiqueta llamada «Cliente»");
    }

    #[test]
    fn an_invalid_path_carries_the_path() {
        let err = AppError::invalid_path(
            Path::new("/no/esta"),
            "path_missing",
            "ya no existe en el disco",
        );
        assert_eq!(
            err.to_string(),
            "ruta inválida: /no/esta: ya no existe en el disco"
        );
        assert_eq!(CommandError::from(err).params["path"], "/no/esta");
    }

    #[test]
    fn a_library_error_travels_with_its_detail() {
        let err = AppError::from(std::io::Error::other("disco lleno"));
        let sent = CommandError::from(err);
        assert_eq!(sent.code, "io");
        assert_eq!(sent.params["detail"], "disco lleno");
        assert_eq!(sent.message, "error de E/S: disco lleno");
    }

    #[test]
    fn it_reaches_the_frontend_as_an_object() {
        let sent =
            CommandError::from(AppError::not_found("tag_not_found", "etiqueta 7").with("id", 7));
        let json = serde_json::to_value(&sent).expect("se serializa");
        assert_eq!(
            json,
            serde_json::json!({
                "code": "tag_not_found",
                "params": { "id": 7 },
                "message": "no encontrado: etiqueta 7",
            })
        );
    }
}
