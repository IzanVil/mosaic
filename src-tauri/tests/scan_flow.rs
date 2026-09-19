//! Test de integración del flujo completo de la Fase 1:
//! registrar una ruta -> escanear -> listar proyectos.
//!
//! Ejercita la API pública de `mosaic_lib` (la misma que usan los comandos
//! Tauri), sin necesidad de levantar una ventana.

use std::fs;
use std::path::Path;

use mosaic_lib::core::now_ts;
use mosaic_lib::core::scanner::{run_full_scan, run_startup_scan};
use mosaic_lib::db::repositories::{projects as projects_repo, scan_paths as scan_paths_repo};
use mosaic_lib::db::Db;

fn mkdir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

fn touch(path: &Path) {
    mkdir(path.parent().unwrap());
    fs::write(path, "").unwrap();
}

/// Construye un árbol con tres proyectos de distinto tipo más ruido alrededor.
fn build_tree(root: &Path) {
    // Proyecto Rust con repositorio Git.
    let rust = root.join("mosaic");
    mkdir(&rust.join(".git"));
    touch(&rust.join("Cargo.toml"));
    touch(&rust.join("src/main.rs"));

    // Proyecto TypeScript sin Git, dentro de una subcarpeta.
    let web = root.join("web/panel");
    touch(&web.join("package.json"));
    touch(&web.join("src/app.ts"));
    touch(&web.join("src/store.ts"));
    // Ruido que el escáner debe ignorar por estar excluido.
    touch(&web.join("node_modules/left-pad/package.json"));

    // Proyecto Python.
    let python = root.join("scripts/etl");
    touch(&python.join("pyproject.toml"));
    touch(&python.join("main.py"));

    // Carpeta sin marcadores: no es un proyecto.
    touch(&root.join("documentos/notas.md"));
}

#[test]
fn scans_a_directory_tree_and_lists_the_projects_it_finds() {
    let tmp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(tmp.path()).unwrap();
    build_tree(&root);

    let db = Db::open_in_memory().unwrap();
    db.with_conn(|conn| scan_paths_repo::add(conn, root.to_str().unwrap(), now_ts()))
        .unwrap();

    let summary = run_full_scan(&db).unwrap();

    assert_eq!(summary.roots_scanned, 1);
    assert_eq!(summary.projects_found, 3);
    assert_eq!(summary.projects_new, 3);
    assert_eq!(summary.projects_missing, 0);
    assert!(!summary.truncated);

    let projects = db.with_conn(projects_repo::list_all).unwrap();
    assert_eq!(projects.len(), 3);

    let etl = projects.iter().find(|p| p.name == "etl").unwrap();
    assert_eq!(etl.path, root.join("scripts/etl").to_str().unwrap());
    assert_eq!(etl.primary_language.as_deref(), Some("Python"));
    assert!(!etl.is_git_repo);
    assert!(!etl.missing);
    assert!(etl.last_seen_at.is_some());

    let mosaic = projects.iter().find(|p| p.name == "mosaic").unwrap();
    assert_eq!(mosaic.primary_language.as_deref(), Some("Rust"));
    assert!(mosaic.is_git_repo);

    let panel = projects.iter().find(|p| p.name == "panel").unwrap();
    assert_eq!(panel.primary_language.as_deref(), Some("TypeScript"));
    assert!(!panel.is_git_repo);

    let scan_path = db.with_conn(scan_paths_repo::list).unwrap().remove(0);
    assert!(scan_path.last_scan_at.is_some());
}

#[test]
fn rescanning_is_idempotent_and_flags_projects_that_disappear() {
    let tmp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(tmp.path()).unwrap();
    build_tree(&root);

    let db = Db::open_in_memory().unwrap();
    db.with_conn(|conn| scan_paths_repo::add(conn, root.to_str().unwrap(), now_ts()))
        .unwrap();

    run_full_scan(&db).unwrap();

    let second = run_full_scan(&db).unwrap();
    assert_eq!(second.projects_found, 3);
    assert_eq!(second.projects_new, 0);
    assert_eq!(second.projects_updated, 0);
    assert_eq!(db.with_conn(projects_repo::count).unwrap(), 3);

    fs::remove_dir_all(root.join("scripts/etl")).unwrap();
    let third = run_full_scan(&db).unwrap();

    assert_eq!(third.projects_found, 2);
    assert_eq!(third.projects_missing, 1);
    assert_eq!(
        db.with_conn(projects_repo::count).unwrap(),
        3,
        "los proyectos ausentes se conservan, no se borran"
    );

    let projects = db.with_conn(projects_repo::list_all).unwrap();
    let etl = projects.iter().find(|p| p.name == "etl").unwrap();
    assert!(etl.missing);
}

/// Prepara una base de datos con el árbol de prueba ya registrado como raíz.
fn db_with_root(root: &Path) -> Db {
    let db = Db::open_in_memory().unwrap();
    db.with_conn(|conn| scan_paths_repo::add(conn, root.to_str().unwrap(), now_ts()))
        .unwrap();
    db
}

#[test]
fn startup_scan_runs_when_the_setting_is_on() {
    let tmp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(tmp.path()).unwrap();
    build_tree(&root);
    let db = db_with_root(&root);

    db.with_conn(|conn| {
        mosaic_lib::config::settings::set_raw(
            conn,
            mosaic_lib::config::settings::KEY_SCAN_ON_STARTUP,
            "true",
        )
    })
    .unwrap();

    let summary = run_startup_scan(&db)
        .unwrap()
        .expect("con el ajuste activado debe escanear");
    assert_eq!(summary.projects_found, 3);
    assert_eq!(db.with_conn(projects_repo::count).unwrap(), 3);
}

#[test]
fn startup_scan_does_nothing_when_the_setting_is_off() {
    let tmp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(tmp.path()).unwrap();
    build_tree(&root);
    let db = db_with_root(&root);

    db.with_conn(|conn| {
        mosaic_lib::config::settings::set_raw(
            conn,
            mosaic_lib::config::settings::KEY_SCAN_ON_STARTUP,
            "false",
        )
    })
    .unwrap();

    assert!(
        run_startup_scan(&db).unwrap().is_none(),
        "con el ajuste desactivado no debe escanear"
    );
    assert_eq!(db.with_conn(projects_repo::count).unwrap(), 0);
}

#[test]
fn startup_scan_is_off_by_default() {
    let tmp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(tmp.path()).unwrap();
    build_tree(&root);
    let db = db_with_root(&root);

    assert!(
        run_startup_scan(&db).unwrap().is_none(),
        "una base de datos nueva no debe escanear al arrancar"
    );
    assert_eq!(db.with_conn(projects_repo::count).unwrap(), 0);
}
