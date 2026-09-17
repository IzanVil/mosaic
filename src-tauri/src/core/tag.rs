use serde::{Deserialize, Serialize};

use crate::errors::{AppError, Result};

pub const MAX_NAME_LEN: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagWithCount {
    #[serde(flatten)]
    pub tag: Tag,
    pub project_count: i64,
}

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
            "3B82F6",
            "#3B82F",
            "#3B82F6A",
            "#GGGGGG",
            "rgb(0,0,0)",
            "",
            "#",
            "#3B8 2F6",
        ] {
            assert!(
                normalize_color(raw).is_err(),
                "«{raw}» no debería aceptarse como color"
            );
        }
    }

    #[test]
    fn normalize_color_accepts_the_short_form_of_nothing() {
        assert!(normalize_color("#FFF").is_err());
    }

    #[test]
    fn name_key_matches_the_sql_index_expression() {
        assert_eq!(name_key("  Cliente  "), "cliente");
        assert_eq!(name_key("CLIENTE"), name_key("cliente"));
    }
}
