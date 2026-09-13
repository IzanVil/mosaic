//! Comandos de consulta de proyectos.

use std::sync::Arc;

use tauri::State;

use crate::core::Project;
use crate::db::repositories::projects as repo;
use crate::db::Db;

/// Devuelve todos los proyectos: primero los fijados, luego por nombre.
#[tauri::command]
pub async fn list_projects(db: State<'_, Arc<Db>>) -> Result<Vec<Project>, String> {
    Ok(db.with_conn(repo::list_all)?)
}

/// Devuelve un proyecto concreto.
#[tauri::command]
pub async fn get_project(db: State<'_, Arc<Db>>, id: i64) -> Result<Project, String> {
    Ok(db.with_conn(|conn| repo::get_by_id(conn, id))?)
}
