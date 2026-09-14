//! Test de integración de la Fase 2: escanear un árbol con repositorios reales
//! y refrescar su estado Git hasta la caché.

use std::fs;
use std::path::Path;

use git2::{IndexAddOption, Repository, RepositoryInitOptions, Signature};
use mosaic_lib::core::git::refresh_all;
use mosaic_lib::core::now_ts;
use mosaic_lib::core::scanner::run_full_scan;
use mosaic_lib::db::repositories::{
    git_status as cache, projects as projects_repo, scan_paths as scan_paths_repo,
};
use mosaic_lib::db::Db;

fn init_repo(path: &Path) -> Repository {
    fs::create_dir_all(path).unwrap();
    let mut options = RepositoryInitOptions::new();
    options.initial_head("main");
    Repository::init_opts(path, &options).unwrap()
}

fn commit_all(repo: &Repository, message: &str) {
    let mut index = repo.index().unwrap();
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();

    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let signature = Signature::now("Test", "test@example.invalid").unwrap();
    let parent = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
    let parents: Vec<&git2::Commit<'_>> = parent.iter().collect();

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )
    .unwrap();
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

/// Un repositorio limpio, uno sucio y un proyecto que no usa Git.
fn build_tree(root: &Path) {
    let limpio = root.join("limpio");
    let repo = init_repo(&limpio);
    write(&limpio.join("Cargo.toml"), "[package]");
    commit_all(&repo, "chore: inicial");

    let sucio = root.join("sucio");
    let repo = init_repo(&sucio);
    write(&sucio.join("package.json"), "{}");
    commit_all(&repo, "feat: arranque del proyecto");
    write(&sucio.join("pendiente.ts"), "// sin commitear");

    write(&root.join("sin-git/pyproject.toml"), "[project]");
}

fn scanned_db(root: &Path) -> Db {
    let db = Db::open_in_memory().unwrap();
    db.with_conn(|conn| scan_paths_repo::add(conn, root.to_str().unwrap(), now_ts()))
        .unwrap();
    run_full_scan(&db).unwrap();
    db
}

#[test]
fn refreshes_the_git_status_of_every_repository_it_finds() {
    let tmp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(tmp.path()).unwrap();
    build_tree(&root);

    let db = scanned_db(&root);
    let summary = refresh_all(&db).unwrap();

    assert_eq!(summary.refreshed, 2, "solo los dos que son repositorios");
    assert_eq!(summary.failed, 0);

    let projects = db.with_conn(projects_repo::list_all).unwrap();
    let entries = db.with_conn(cache::list_all).unwrap();
    assert_eq!(entries.len(), 2);

    let id_of = |name: &str| projects.iter().find(|p| p.name == name).unwrap().id;

    let limpio = entries
        .iter()
        .find(|entry| entry.project_id == id_of("limpio"))
        .unwrap();
    assert_eq!(limpio.status.branch.as_deref(), Some("main"));
    assert!(!limpio.status.is_dirty);
    assert_eq!(
        limpio.status.last_commit_msg.as_deref(),
        Some("chore: inicial")
    );
    assert_eq!(limpio.status.ahead, None, "sin upstream configurado");

    let sucio = entries
        .iter()
        .find(|entry| entry.project_id == id_of("sucio"))
        .unwrap();
    assert!(sucio.status.is_dirty);
    assert_eq!(
        sucio.status.last_commit_msg.as_deref(),
        Some("feat: arranque del proyecto")
    );

    // El proyecto sin Git no entra en la caché.
    assert!(!entries
        .iter()
        .any(|entry| entry.project_id == id_of("sin-git")));
}

#[test]
fn refreshing_twice_updates_the_cache_without_duplicating_it() {
    let tmp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(tmp.path()).unwrap();
    build_tree(&root);

    let db = scanned_db(&root);
    refresh_all(&db).unwrap();

    // Se limpia el repositorio sucio y se vuelve a refrescar.
    fs::remove_file(root.join("sucio/pendiente.ts")).unwrap();
    let segundo = refresh_all(&db).unwrap();

    assert_eq!(segundo.refreshed, 2);
    let entries = db.with_conn(cache::list_all).unwrap();
    assert_eq!(entries.len(), 2, "una fila por proyecto");
    assert!(entries.iter().all(|entry| !entry.status.is_dirty));
}

#[test]
fn a_repository_that_disappears_drops_out_of_the_cache() {
    let tmp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(tmp.path()).unwrap();
    build_tree(&root);

    let db = scanned_db(&root);
    refresh_all(&db).unwrap();
    assert_eq!(db.with_conn(cache::list_all).unwrap().len(), 2);

    // Se borra el repositorio pero el proyecto sigue registrado: el refresco
    // no puede leerlo y retira su entrada en vez de dejar datos rancios.
    fs::remove_dir_all(root.join("sucio")).unwrap();
    let summary = refresh_all(&db).unwrap();

    assert_eq!(summary.refreshed, 1);
    assert_eq!(summary.failed, 1);
    assert_eq!(db.with_conn(cache::list_all).unwrap().len(), 1);
}
