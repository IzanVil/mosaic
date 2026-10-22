//! Un proyecto. Los campos van en `snake_case`, como las columnas SQL y
//! `src/lib/types/index.ts`, que es su espejo literal.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::{AppError, Result};

/// Tope de las notas, en caracteres: solo para que un pegado accidental no llene la base.
pub const MAX_NOTES_CHARS: usize = 20_000;

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
    /// Nombre visible del proyecto: su carpeta, o la ruta entera si no tiene componente final.
    pub fn name_from_path(path: &Path) -> String {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned())
    }
}

/// Prepara las notas para guardarlas: vacías o solo espacios son `NULL`; lo demás, tal cual.
pub fn normalize_notes(raw: &str) -> Result<Option<String>> {
    if raw.trim().is_empty() {
        return Ok(None);
    }
    if raw.chars().count() > MAX_NOTES_CHARS {
        return Err(AppError::validation(
            "notes_too_long",
            format!("las notas no pueden pasar de {MAX_NOTES_CHARS} caracteres"),
        )
        .with("max", MAX_NOTES_CHARS));
    }
    // Sin recortar: el editor autoguarda al teclear y se comería el espacio recién escrito.
    Ok(Some(raw.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_notes_turns_blank_text_into_null() {
        assert_eq!(normalize_notes("").unwrap(), None);
        assert_eq!(normalize_notes("  \n\t ").unwrap(), None);
    }

    #[test]
    fn normalize_notes_keeps_the_text_exactly_as_typed() {
        let typed = "  pendiente: migrar a Tauri 3 \n";
        assert_eq!(normalize_notes(typed).unwrap().as_deref(), Some(typed));
    }

    #[test]
    fn normalize_notes_counts_characters_not_bytes() {
        // 20.000 eñes son 40.000 bytes, pero siguen dentro del tope.
        let at_limit = "ñ".repeat(MAX_NOTES_CHARS);
        assert!(normalize_notes(&at_limit).is_ok());

        let over = "a".repeat(MAX_NOTES_CHARS + 1);
        assert!(matches!(
            normalize_notes(&over),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn name_from_path_uses_folder_name() {
        assert_eq!(
            DiscoveredProject::name_from_path(Path::new("/proyectos/mosaic")),
            "mosaic"
        );
    }

    #[test]
    fn name_from_path_falls_back_to_full_path() {
        assert_eq!(DiscoveredProject::name_from_path(Path::new("/")), "/");
    }
}
