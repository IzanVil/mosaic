use std::sync::Arc;

use tauri::State;

use crate::config::settings::{self, KEY_VIEW_STATE};
use crate::db::Db;
use crate::errors::AppError;

const MAX_VIEW_STATE_BYTES: usize = 64 * 1024;

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
