//! Los datos de Mosaic y sus reglas de validación. Solo depende de los errores,
//! así que lo usan a la vez `core`, que orquesta, y `db`, que guarda.

pub mod git;
pub mod project;
pub mod tag;

pub use git::GitStatus;
pub use project::{DiscoveredProject, Project};
pub use tag::{Tag, TagWithCount};
