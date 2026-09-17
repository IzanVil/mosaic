use std::sync::Arc;

use tauri::State;

use crate::core::Project;
use crate::db::repositories::projects::{self as repo, ProjectWithTags};
use crate::db::Db;

#[tauri::command]
pub async fn list_projects(db: State<'_, Arc<Db>>) -> Result<Vec<Project>, String> {
    let projects = db.with_conn(repo::list_all)?;
    tracing::debug!(count = projects.len(), "list_projects");
    Ok(projects)
}

#[tauri::command]
pub async fn list_projects_with_tags(
    db: State<'_, Arc<Db>>,
) -> Result<Vec<ProjectWithTags>, String> {
    let projects = db.with_conn(repo::list_all_with_tags)?;
    tracing::debug!(count = projects.len(), "list_projects_with_tags");
    Ok(projects)
}

#[tauri::command]
pub async fn get_project(db: State<'_, Arc<Db>>, id: i64) -> Result<Project, String> {
    Ok(db.with_conn(|conn| repo::get_by_id(conn, id))?)
}

#[tauri::command]
pub async fn set_project_pinned(
    db: State<'_, Arc<Db>>,
    id: i64,
    pinned: bool,
) -> Result<(), String> {
    db.with_conn(|conn| repo::set_pinned(conn, id, pinned))?;
    Ok(())
}
