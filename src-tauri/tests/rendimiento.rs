//! Mediciones de rendimiento del backend con miles de proyectos.
//!
//! No es un test de verdad: está ignorado y solo imprime tiempos. Se lanza con
//! los proyectos de `scripts/rendimiento/generar.py`:
//!
//! ```bash
//! MOSAIC_RENDIMIENTO=/ruta/proyectos cargo test --release --test rendimiento -- --ignored --nocapture
//! ```

use std::path::Path;
use std::time::Instant;

use mosaic_lib::config::settings;
use mosaic_lib::core::{git, scanner};
use mosaic_lib::db::repositories::{projects, scan_paths};
use mosaic_lib::db::Db;

fn medir(base: &Path, raices: &[&str], tope: Option<usize>) {
    let dir = tempfile::tempdir().unwrap();
    let db = Db::open(&dir.path().join("mosaic.db")).unwrap();
    db.with_conn(|c| {
        for r in raices {
            scan_paths::add(c, base.join(r).to_str().unwrap(), 1)?;
        }
        if let Some(tope) = tope {
            let mut s = settings::load(c)?;
            s.max_entries_per_scan = tope;
            settings::save(c, &s)?;
        }
        Ok(())
    })
    .unwrap();

    let primero = scanner::run_full_scan(&db).unwrap();
    let segundo = scanner::run_full_scan(&db).unwrap();
    let git = git::refresh_all(&db).unwrap();

    let mut lectura = Vec::new();
    let mut json = 0;
    for _ in 0..5 {
        let t = Instant::now();
        let lista = db.with_conn(projects::list_all_with_tags).unwrap();
        json = serde_json::to_string(&lista).unwrap().len();
        lectura.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    lectura.sort_by(|a, b| a.partial_cmp(b).unwrap());

    println!(
        "RAICES {:?} tope {:?}: {} proyectos, {} entradas, recortado {} | escaneo {} ms, reescaneo {} ms | git {} repos en {} ms | lista+json {:.1} ms (mediana), {} KB",
        raices,
        tope.unwrap_or(settings::DEFAULT_MAX_ENTRIES_PER_SCAN),
        primero.projects_found,
        primero.entries_visited,
        primero.truncated,
        primero.elapsed_ms,
        segundo.elapsed_ms,
        git.refreshed,
        git.elapsed_ms,
        lectura[2],
        json / 1024,
    );
}

#[test]
#[ignore]
fn rendimiento_con_miles_de_proyectos() {
    let base = std::env::var("MOSAIC_RENDIMIENTO").expect("falta MOSAIC_RENDIMIENTO");
    let base = Path::new(&base);
    medir(base, &["a"], None);
    medir(base, &["a", "b"], None);
    medir(base, &["a", "b", "c"], None);
    medir(base, &["a", "b", "c"], Some(500_000));
}
