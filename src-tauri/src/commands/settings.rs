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
use crate::db::Db;
use crate::errors::AppError;
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
/// [`settings::load`] con el resto de claves.
#[tauri::command]
pub async fn get_view_state(db: State<'_, Arc<Db>>) -> Result<Option<String>, String> {
    let raw = db.with_conn(|conn| settings::get_raw(conn, KEY_VIEW_STATE))?;

    let Some(raw) = raw else {
        return Ok(None);
    };

    match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(_) => Ok(Some(raw)),
        Err(err) => {
            tracing::warn!(error = %err, "vista guardada ilegible: se usan los filtros por defecto");
            Ok(None)
        }
    }
}

/// Guarda la vista actual. El frontend llama con debounce, no en cada tecla.
#[tauri::command]
pub async fn set_view_state(db: State<'_, Arc<Db>>, json: String) -> Result<(), String> {
    if json.len() > MAX_VIEW_STATE_BYTES {
        return Err(AppError::Validation(format!(
            "la vista guardada no puede pasar de {MAX_VIEW_STATE_BYTES} bytes"
        ))
        .into());
    }

    serde_json::from_str::<serde_json::Value>(&json).map_err(|err| {
        AppError::Validation(format!("la vista guardada no es JSON válido: {err}"))
    })?;

    db.with_conn(|conn| settings::set_raw(conn, KEY_VIEW_STATE, &json))?;
    Ok(())
}

/// Devuelve los ajustes avanzados con sus valores de fábrica y sus límites.
#[tauri::command]
pub async fn get_advanced_settings(db: State<'_, Arc<Db>>) -> Result<AdvancedSettingsView, String> {
    let current = AdvancedSettings::from(&db.with_conn(settings::load)?);
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
) -> Result<AdvancedSettings, String> {
    let validos = validate_advanced(settings)?;
    let antes = db.with_conn(settings::load)?.git_refresh_interval_minutes;
    db.with_conn(|conn| settings::save_advanced(conn, &validos))?;

    if validos.git_refresh_interval_minutes != antes {
        signal.0.notify_one();
    }
    tracing::debug!(?validos, "set_advanced_settings");
    Ok(validos)
}
