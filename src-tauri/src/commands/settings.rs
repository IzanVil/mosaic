//! Comandos de estado persistido de la interfaz.
//!
//! Solo exponen la clave `ui.view_state`, no la tabla `settings` entera: un
//! `get_setting`/`set_setting` genérico dejaría al frontend escribir cualquier
//! ajuste del escáner o de Git sin pasar por su validación.

use std::sync::Arc;

use tauri::State;

use crate::config::settings::{self, KEY_VIEW_STATE};
use crate::db::Db;
use crate::errors::AppError;

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
