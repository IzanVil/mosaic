//! Test de integración de la Fase 4: escanear un árbol real, etiquetar los
//! proyectos descubiertos y comprobar que el tablero los recibe con sus
//! etiquetas, y que borrar una etiqueta no se lleva nada más por delante.

use std::fs;
use std::path::Path;

use mosaic_lib::core::scanner::run_full_scan;
use mosaic_lib::db::repositories::{
    project_tags, projects as projects_repo, scan_paths as scan_paths_repo, tags as tags_repo,
};
use mosaic_lib::db::Db;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

/// Tres proyectos detectables por su fichero marcador.
fn build_tree(root: &Path) {
    write(&root.join("uno/Cargo.toml"), "[package]");
    write(&root.join("dos/package.json"), "{}");
    write(&root.join("tres/pyproject.toml"), "[project]");
}

/// Escanea `root` y devuelve la base de datos ya poblada.
fn scanned_db(root: &Path) -> Db {
    let db = Db::open_in_memory().unwrap();
    db.with_conn(|conn| scan_paths_repo::add(conn, root.to_str().unwrap(), 0))
        .unwrap();
    let summary = run_full_scan(&db).unwrap();
    assert_eq!(summary.projects_found, 3, "el árbol tiene tres proyectos");
    db
}

#[test]
fn tagging_projects_survives_a_rescan_and_the_tag_being_deleted() {
    let tmp = tempfile::tempdir().unwrap();
    build_tree(tmp.path());
    let db = scanned_db(tmp.path());

    let ids: Vec<i64> = db
        .with_conn(projects_repo::list_all)
        .unwrap()
        .into_iter()
        .map(|p| p.id)
        .collect();

    // Una etiqueta en los tres proyectos y otra solo en el primero.
    let cliente = db
        .with_conn(|conn| tags_repo::create(conn, "Cliente", "#3b82f6", 1_000))
        .unwrap();
    let urgente = db
        .with_conn(|conn| tags_repo::create(conn, "urgente", "#EF4444", 1_000))
        .unwrap();
    for &id in &ids {
        db.with_conn(|conn| project_tags::assign(conn, id, cliente.id))
            .unwrap();
    }
    db.with_conn(|conn| project_tags::assign(conn, ids[0], urgente.id))
        .unwrap();

    let rows = db.with_conn(projects_repo::list_all_with_tags).unwrap();
    assert_eq!(rows.len(), 3);
    for row in &rows {
        assert!(
            row.tags.iter().any(|t| t.id == cliente.id),
            "{} debería llevar «Cliente»",
            row.project.name
        );
    }
    let primero = rows.iter().find(|r| r.project.id == ids[0]).unwrap();
    let nombres: Vec<&str> = primero.tags.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(nombres, vec!["Cliente", "urgente"]);

    // El conteo del sidebar sale de la misma consulta.
    let listado = db.with_conn(tags_repo::list_all).unwrap();
    let conteos: Vec<(&str, i64)> = listado
        .iter()
        .map(|t| (t.tag.name.as_str(), t.project_count))
        .collect();
    assert_eq!(conteos, vec![("Cliente", 3), ("urgente", 1)]);

    // Un reescaneo no altera las asignaciones ni los metadatos del proyecto.
    let antes = db
        .with_conn(|conn| projects_repo::get_by_id(conn, ids[0]))
        .unwrap();
    run_full_scan(&db).unwrap();
    let despues = db
        .with_conn(|conn| projects_repo::get_by_id(conn, ids[0]))
        .unwrap();
    assert_eq!(
        despues.updated_at, antes.updated_at,
        "un reescaneo sin cambios no mueve updated_at"
    );
    assert_eq!(
        db.with_conn(|conn| project_tags::list_tags_for_project(conn, ids[0]))
            .unwrap()
            .len(),
        2
    );

    // Borrar la etiqueta la retira de los proyectos, que siguen ahí.
    db.with_conn(|conn| tags_repo::delete(conn, cliente.id))
        .unwrap();
    let rows = db.with_conn(projects_repo::list_all_with_tags).unwrap();
    assert_eq!(rows.len(), 3, "borrar una etiqueta no borra proyectos");
    assert!(
        rows.iter()
            .all(|r| r.tags.iter().all(|t| t.id != cliente.id)),
        "la etiqueta borrada no puede seguir apareciendo"
    );
    assert_eq!(
        db.with_conn(|conn| project_tags::list_tags_for_project(conn, ids[0]))
            .unwrap()
            .len(),
        1,
        "«urgente» sobrevive al borrado de «Cliente»"
    );
}

#[test]
fn tags_survive_a_project_going_missing() {
    let tmp = tempfile::tempdir().unwrap();
    build_tree(tmp.path());
    let db = scanned_db(tmp.path());

    let uno = db
        .with_conn(|conn| {
            projects_repo::find_by_path(conn, tmp.path().join("uno").to_str().unwrap())
        })
        .unwrap()
        .unwrap();
    let archivado = db
        .with_conn(|conn| tags_repo::create(conn, "archivado", "#64748B", 0))
        .unwrap();
    db.with_conn(|conn| project_tags::assign(conn, uno.id, archivado.id))
        .unwrap();

    // Desaparece del disco: el escáner lo marca ausente, no lo borra.
    fs::remove_dir_all(tmp.path().join("uno")).unwrap();
    let summary = run_full_scan(&db).unwrap();
    assert_eq!(summary.projects_missing, 1);

    let row = db
        .with_conn(projects_repo::list_all_with_tags)
        .unwrap()
        .into_iter()
        .find(|r| r.project.id == uno.id)
        .expect("el proyecto ausente sigue en el listado");
    assert!(row.project.missing);
    assert_eq!(
        row.tags,
        vec![archivado],
        "perder la carpeta no puede perder las etiquetas"
    );
}
