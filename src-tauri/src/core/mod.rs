//! Lógica de negocio de Mosaic, independiente de Tauri.

pub mod git;
pub mod launcher;
pub mod project;
pub mod scanner;
pub mod tag;

pub use project::{DiscoveredProject, Project};
pub use tag::{Tag, TagWithCount};

/// Marca de tiempo actual en segundos desde el epoch Unix.
///
/// Si el reloj del sistema estuviese antes del epoch se devuelve `0`, que es
/// preferible a propagar un error por algo que no puede corregir el usuario.
pub fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
