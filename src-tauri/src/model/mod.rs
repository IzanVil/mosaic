//! Los datos de Mosaic y sus reglas de validación.
//!
//! No depende de ninguna otra parte del crate salvo de los errores, así que
//! lo pueden usar a la vez `core`, que orquesta, y `db`, que guarda.

pub mod git;
pub mod project;
pub mod tag;

pub use git::GitStatus;
pub use project::{DiscoveredProject, Project};
pub use tag::{Tag, TagWithCount};
