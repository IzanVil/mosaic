//! Modelo de dominio de las etiquetas y sus reglas de validación.
//!
//! La validación vive aquí y no en el repositorio para que `db/` se quede solo
//! con SQL y para que estas reglas queden cubiertas por los tests de `core/`.
//!
//! Como el resto del dominio, los campos se serializan en `snake_case` para
//! que `src/lib/types/index.ts` sea un espejo literal de estos structs.

use serde::{Deserialize, Serialize};

use crate::errors::{AppError, Result};

/// Longitud máxima del nombre de una etiqueta, en caracteres (no bytes).
pub const MAX_NAME_LEN: usize = 32;

/// Etiqueta tal y como está registrada en la base de datos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    /// Nombre tal y como lo escribió el usuario, con sus mayúsculas.
    pub name: String,
    /// Color en formato `#RRGGBB`, siempre en mayúsculas.
    pub color: String,
    pub created_at: i64,
}

/// Etiqueta con el número de proyectos que la tienen asignada.
///
/// Se serializa aplanada, igual que
/// [`crate::db::repositories::git_status::GitStatusEntry`], así que el frontend
/// recibe `project_count` al mismo nivel que los campos de [`Tag`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagWithCount {
    #[serde(flatten)]
    pub tag: Tag,
    pub project_count: i64,
}

/// Normaliza y valida el nombre de una etiqueta.
///
/// Recorta los espacios de los extremos en silencio y exige que lo que queda
/// tenga entre 1 y [`MAX_NAME_LEN`] caracteres. Los espacios interiores se
/// permiten ("cliente grande"); los caracteres de control no, porque
/// romperían el renderizado de los chips y no son escribibles a propósito.
pub fn validate_name(raw: &str) -> Result<String> {
    let name = raw.trim();

    if name.is_empty() {
        return Err(AppError::Validation(
            "el nombre de la etiqueta no puede estar vacío".into(),
        ));
    }

    let len = name.chars().count();
    if len > MAX_NAME_LEN {
        return Err(AppError::Validation(format!(
            "el nombre de la etiqueta no puede pasar de {MAX_NAME_LEN} caracteres (tiene {len})"
        )));
    }

    if name.chars().any(|c| c.is_control()) {
        return Err(AppError::Validation(
            "el nombre de la etiqueta no puede contener caracteres de control".into(),
        ));
    }

    Ok(name.to_string())
}

/// Normaliza y valida un color de etiqueta.
///
/// Acepta `#RRGGBB` en cualquier caja y lo devuelve en mayúsculas, para que la
/// base de datos no acumule las dos formas del mismo color y el frontend pueda
/// comparar contra la paleta con una igualdad de cadenas.
pub fn normalize_color(raw: &str) -> Result<String> {
    let color = raw.trim();

    let invalid = || {
        AppError::Validation(format!(
            "el color debe tener el formato #RRGGBB, no «{color}»"
        ))
    };

    let hex = color.strip_prefix('#').ok_or_else(invalid)?;
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(invalid());
    }

    Ok(format!("#{}", hex.to_ascii_uppercase()))
}

/// Clave de comparación para la unicidad insensible a mayúsculas.
///
/// Debe coincidir con lo que calcula el índice `idx_tags_name_ci` de la
/// migración 002, que es `lower(name)`.
pub fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_name_trims_silently() {
        assert_eq!(validate_name("  cliente  ").unwrap(), "cliente");
    }

    #[test]
    fn validate_name_keeps_the_user_casing_and_inner_spaces() {
        assert_eq!(
            validate_name("Cliente Grande").unwrap(),
            "Cliente Grande",
            "ni se baja a minúsculas ni se colapsan los espacios interiores"
        );
    }

    #[test]
    fn validate_name_rejects_empty_and_whitespace_only() {
        assert!(validate_name("").is_err());
        assert!(validate_name("   ").is_err());
        assert!(validate_name("\t\n").is_err());
    }

    #[test]
    fn validate_name_counts_characters_not_bytes() {
        // 32 caracteres multibyte: 64 bytes, pero un nombre válido.
        let name = "ñ".repeat(MAX_NAME_LEN);
        assert_eq!(validate_name(&name).unwrap().chars().count(), MAX_NAME_LEN);

        let too_long = "ñ".repeat(MAX_NAME_LEN + 1);
        assert!(validate_name(&too_long).is_err());
    }

    #[test]
    fn validate_name_measures_after_trimming() {
        let padded = format!("  {}  ", "a".repeat(MAX_NAME_LEN));
        assert!(
            validate_name(&padded).is_ok(),
            "los espacios recortados no deben contar para el límite"
        );
    }

    #[test]
    fn validate_name_rejects_control_characters() {
        assert!(validate_name("cli\u{0000}ente").is_err());
        assert!(validate_name("cli\u{001F}ente").is_err());
        assert!(validate_name("cli\u{007F}ente").is_err());
        assert!(
            validate_name("cli\nente").is_err(),
            "un salto de línea interior también es un carácter de control"
        );
    }

    #[test]
    fn normalize_color_uppercases_the_hex() {
        assert_eq!(normalize_color("#3b82f6").unwrap(), "#3B82F6");
        assert_eq!(normalize_color("  #3B82F6  ").unwrap(), "#3B82F6");
    }

    #[test]
    fn normalize_color_rejects_anything_that_is_not_rrggbb() {
        for raw in [
            "3B82F6",     // sin almohadilla
            "#3B82F",     // cinco dígitos
            "#3B82F6A",   // siete dígitos
            "#GGGGGG",    // no hexadecimal
            "rgb(0,0,0)", // otro formato
            "",
            "#",
            "#3B8 2F6", // espacio interior
        ] {
            assert!(
                normalize_color(raw).is_err(),
                "«{raw}» no debería aceptarse como color"
            );
        }
    }

    #[test]
    fn normalize_color_accepts_the_short_form_of_nothing() {
        // #RGB es válido en CSS pero no aquí: el esquema y el frontend asumen
        // siete caracteres para poder concatenar el alfa.
        assert!(normalize_color("#FFF").is_err());
    }

    #[test]
    fn name_key_matches_the_sql_index_expression() {
        assert_eq!(name_key("  Cliente  "), "cliente");
        assert_eq!(name_key("CLIENTE"), name_key("cliente"));
    }
}
