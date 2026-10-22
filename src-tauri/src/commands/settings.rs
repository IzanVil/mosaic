//! Comandos de estado persistido de la interfaz.
//!
//! Exponen la clave `ui.view_state` y los cinco ajustes avanzados, no la tabla
//! `settings` entera: un `get_setting`/`set_setting` genérico dejaría al
//! frontend escribir cualquier ajuste sin pasar por su validación.

use std::sync::Arc;

use tauri::State;

use crate::config::settings::{
    self, validate_advanced, AdvancedSettings, AdvancedSettingsView, KEY_VIEW_STATE, LIMITS,
};
use crate::db::repositories::settings as settings_repo;
use crate::db::Db;
use crate::errors::{AppError, CommandError};
use crate::GitRefreshSignal;

/// Tope de tamaño del JSON de la vista.
///
/// La vista guardada son unos cientos de bytes: query, ids de etiquetas y
/// orden. El tope evita que un error del frontend convierta la tabla de
/// ajustes en un vertedero.
const MAX_VIEW_STATE_BYTES: usize = 64 * 1024;

/// Devuelve la última vista guardada (búsqueda, filtros, orden) como JSON.
///
/// Devuelve `null` si nunca se guardó o si lo guardado ya no es JSON válido:
/// la vista es una comodidad, así que ante un valor corrupto se arranca con
/// los filtros por defecto en lugar de fallar. Es la misma política que aplica
/// [`settings_repo::load`] con el resto de claves.
#[tauri::command]
pub async fn get_view_state(db: State<'_, Arc<Db>>) -> Result<Option<String>, CommandError> {
    let raw = db.with_conn(|conn| settings_repo::get_raw(conn, KEY_VIEW_STATE))?;
    Ok(raw.and_then(readable_view_state))
}

/// La vista guardada si sigue siendo JSON; si no, `None` y un aviso en el log.
fn readable_view_state(raw: String) -> Option<String> {
    match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(_) => Some(raw),
        Err(err) => {
            tracing::warn!(error = %err, "vista guardada ilegible: se usan los filtros por defecto");
            None
        }
    }
}

/// Guarda la vista actual. El frontend llama con debounce, no en cada tecla.
#[tauri::command]
pub async fn set_view_state(db: State<'_, Arc<Db>>, json: String) -> Result<(), CommandError> {
    validate_view_state(&json)?;
    db.with_conn(|conn| settings_repo::set_raw(conn, KEY_VIEW_STATE, &json))?;
    Ok(())
}

/// Comprueba que la vista quepa en [`MAX_VIEW_STATE_BYTES`] y sea JSON.
fn validate_view_state(json: &str) -> Result<(), AppError> {
    if json.len() > MAX_VIEW_STATE_BYTES {
        return Err(AppError::validation(
            "view_state_too_big",
            format!("la vista guardada no puede pasar de {MAX_VIEW_STATE_BYTES} bytes"),
        )
        .with("max", MAX_VIEW_STATE_BYTES));
    }

    serde_json::from_str::<serde_json::Value>(json).map_err(|err| {
        AppError::validation(
            "view_state_invalid",
            format!("la vista guardada no es JSON válido: {err}"),
        )
        .with("detail", err.to_string())
    })?;
    Ok(())
}

/// Devuelve los ajustes avanzados con sus valores de fábrica y sus límites.
#[tauri::command]
pub async fn get_advanced_settings(
    db: State<'_, Arc<Db>>,
) -> Result<AdvancedSettingsView, CommandError> {
    let current = AdvancedSettings::from(&db.with_conn(settings_repo::load)?);
    Ok(AdvancedSettingsView {
        current,
        defaults: AdvancedSettings::from(&settings::Settings::default()),
        limits: LIMITS,
    })
}

/// Valida y guarda los ajustes avanzados, y devuelve lo que ha quedado
/// guardado (con los nombres de carpeta ya recortados).
///
/// Si cambia el intervalo de Git, despierta la tarea de refresco para que el
/// cambio se note ya y no al acabar la espera en curso.
#[tauri::command]
pub async fn set_advanced_settings(
    db: State<'_, Arc<Db>>,
    signal: State<'_, Arc<GitRefreshSignal>>,
    settings: AdvancedSettings,
) -> Result<AdvancedSettings, CommandError> {
    let validos = validate_advanced(settings)?;
    let antes = db
        .with_conn(settings_repo::load)?
        .git_refresh_interval_minutes;
    db.with_conn(|conn| settings_repo::save_advanced(conn, &validos))?;

    if validos.git_refresh_interval_minutes != antes {
        signal.0.notify_one();
    }
    tracing::debug!(?validos, "set_advanced_settings");
    Ok(validos)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un JSON de texto que ocupa exactamente `bytes` bytes.
    fn json_of(bytes: usize) -> String {
        format!("\"{}\"", "a".repeat(bytes - 2))
    }

    #[test]
    fn a_view_right_at_the_limit_is_saved() {
        assert!(validate_view_state(&json_of(MAX_VIEW_STATE_BYTES)).is_ok());
    }

    #[test]
    fn a_view_one_byte_over_the_limit_is_rejected_and_says_the_limit() {
        let err = validate_view_state(&json_of(MAX_VIEW_STATE_BYTES + 1)).unwrap_err();
        assert_eq!(err.code(), "view_state_too_big");
        assert_eq!(CommandError::from(err).params["max"], MAX_VIEW_STATE_BYTES);
    }

    #[test]
    fn the_limit_counts_bytes_not_characters() {
        // «ñ» ocupa dos bytes: medio tope de eñes ya se pasa por las comillas.
        let json = format!("\"{}\"", "ñ".repeat(MAX_VIEW_STATE_BYTES / 2));
        assert_eq!(
            validate_view_state(&json).unwrap_err().code(),
            "view_state_too_big"
        );
    }

    #[test]
    fn a_view_that_is_not_json_is_rejected_with_the_reason() {
        let err = validate_view_state(r#"{"query": "#).unwrap_err();
        assert_eq!(err.code(), "view_state_invalid");
        assert!(CommandError::from(err).params["detail"].is_string());
    }

    #[test]
    fn a_corrupt_saved_view_is_ignored_and_a_valid_one_comes_back_untouched() {
        assert_eq!(readable_view_state("{roto".into()), None);
        let guardada = r#"{"query":"api","density":"compacto"}"#;
        assert_eq!(
            readable_view_state(guardada.into()).as_deref(),
            Some(guardada)
        );
    }
}
