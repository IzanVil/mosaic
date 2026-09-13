//! Modelo de dominio de un proyecto.
//!
//! Los campos se serializan en `snake_case` (el comportamiento por defecto de
//! serde) para que `src/lib/types/index.ts` sea un espejo literal de estos
//! structs y de las columnas SQL.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Proyecto tal y como está registrado en la base de datos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    /// Nombre de la carpeta del proyecto.
    pub name: String,
    /// Ruta absoluta y canónica. Es la clave natural del proyecto.
    pub path: String,
    pub is_git_repo: bool,
    /// Lenguaje principal según la heurística de [`crate::core::scanner`].
    pub primary_language: Option<String>,
    pub last_opened_at: Option<i64>,
    pub pinned: bool,
    pub notes: Option<String>,
    /// `true` si el último escaneo de su raíz no encontró la carpeta en disco.
    pub missing: bool,
    /// Última vez que el escáner vio el proyecto en disco.
    pub last_seen_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Proyecto recién descubierto en disco, todavía sin identidad en la base de datos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredProject {
    pub name: String,
    pub path: PathBuf,
    pub is_git_repo: bool,
    pub primary_language: Option<String>,
    /// Ficheros marcadores que delataron la carpeta, en el orden de
    /// [`crate::core::scanner::PROJECT_MARKERS`].
    pub markers: Vec<String>,
}

impl DiscoveredProject {
    /// Deriva el nombre visible de un proyecto a partir de su ruta.
    ///
    /// Usa el nombre de la carpeta; para rutas sin componente final (`/`)
    /// recurre a la representación completa de la ruta.
    pub fn name_from_path(path: &Path) -> String {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_from_path_uses_folder_name() {
        assert_eq!(
            DiscoveredProject::name_from_path(Path::new("/home/izan/code/mosaic")),
            "mosaic"
        );
    }

    #[test]
    fn name_from_path_falls_back_to_full_path() {
        assert_eq!(DiscoveredProject::name_from_path(Path::new("/")), "/");
    }
}
