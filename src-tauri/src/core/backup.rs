//! Exportar e importar la configuración y el trabajo.
//!
//! Importar suma, nunca borra:
//! - rutas de escaneo: se añaden las que faltan y existen en este equipo;
//! - carpetas excluidas: se unen; el resto de ajustes toma el valor del fichero;
//! - etiquetas: se crean si no hay otra con el mismo nombre; si la hay, conserva su color;
//! - proyectos: se buscan por ruta y nunca se crean; se fija pero no se desfija,
//!   se etiqueta pero no se desetiqueta, y las notas de aquí nunca se pisan.
//!
//! La vista guardada no viaja: sus filtros apuntan a ids de etiqueta locales.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::config::settings::{
    self, validate_advanced, AdvancedSettings, KEY_PREFERRED_IDE, KEY_PREFERRED_TERMINAL,
};
use crate::core::project::normalize_notes;
use crate::core::tag;
use crate::db::repositories::{project_tags, projects, scan_paths, tags};
use crate::errors::{AppError, Result};

/// Marca que identifica un fichero como copia de Mosaic.
pub const FORMAT_TAG: &str = "mosaic-backup";
/// Versión del formato de la copia.
pub const FORMAT_VERSION: u32 = 1;
/// Tamaño máximo de un fichero de copia.
pub const MAX_BACKUP_BYTES: u64 = 32 * 1024 * 1024;

/// Fichero de copia completo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Backup {
    pub format: String,
    pub version: u32,
    pub exported_at: i64,
    pub app_version: String,
    pub settings: BackupSettings,
    pub scan_paths: Vec<BackupScanPath>,
    /// `None` si se exportó sin etiquetas ni notas.
    pub work: Option<BackupWork>,
}

/// Ajustes de la copia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupSettings {
    #[serde(flatten)]
    pub advanced: AdvancedSettings,
    pub preferred_ide: String,
    pub preferred_terminal: String,
}

/// Ruta de escaneo de la copia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupScanPath {
    pub path: String,
    pub enabled: bool,
}

/// Etiquetas y proyectos de la copia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupWork {
    pub tags: Vec<BackupTag>,
    /// Solo los fijados, con notas o con etiquetas.
    pub projects: Vec<BackupProject>,
}

/// Etiqueta de la copia, identificada por nombre.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupTag {
    pub name: String,
    pub color: String,
}

/// Proyecto de la copia, identificado por ruta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupProject {
    pub path: String,
    pub pinned: bool,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

/// Recuento de una exportación.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportSummary {
    pub scan_paths: usize,
    pub tags: usize,
    pub projects: usize,
}

/// Lo que hará una importación, o lo que hizo.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportPlan {
    pub exported_at: i64,
    pub app_version: String,
    pub has_work: bool,
    pub settings_changed: Vec<String>,
    pub git_interval_changed: bool,
    pub scan_paths_added: usize,
    /// No existen en este equipo y no se añaden.
    pub scan_paths_missing: Vec<String>,
    pub tags_added: usize,
    pub tags_existing: usize,
    pub projects_matched: usize,
    pub projects_not_found: usize,
    pub pins_added: usize,
    pub assignments_added: usize,
    pub notes_added: usize,
    /// Nombres de proyecto cuyas notas difieren; se quedan las de aquí.
    pub notes_conflicts: Vec<String>,
}

impl ImportPlan {
    /// Si importar no cambiaría nada.
    pub fn is_empty(&self) -> bool {
        self.settings_changed.is_empty()
            && self.scan_paths_added == 0
            && self.tags_added == 0
            && self.pins_added == 0
            && self.assignments_added == 0
            && self.notes_added == 0
    }
}

/// Construye la copia desde la base de datos.
pub fn build_backup(
    conn: &Connection,
    include_work: bool,
    now: i64,
    app_version: &str,
) -> Result<Backup> {
    let loaded = settings::load(conn)?;
    let backup_settings = BackupSettings {
        advanced: AdvancedSettings::from(&loaded),
        preferred_ide: loaded.preferred_ide,
        preferred_terminal: loaded.preferred_terminal,
    };

    let backup_paths = scan_paths::list(conn)?
        .into_iter()
        .map(|p| BackupScanPath {
            path: p.path,
            enabled: p.enabled,
        })
        .collect();

    let work = if include_work {
        Some(build_work(conn)?)
    } else {
        None
    };

    Ok(Backup {
        format: FORMAT_TAG.to_string(),
        version: FORMAT_VERSION,
        exported_at: now,
        app_version: app_version.to_string(),
        settings: backup_settings,
        scan_paths: backup_paths,
        work,
    })
}

fn build_work(conn: &Connection) -> Result<BackupWork> {
    let backup_tags = tags::list_all(conn)?
        .into_iter()
        .map(|t| BackupTag {
            name: t.tag.name,
            color: t.tag.color,
        })
        .collect();

    let mut grouped = project_tags::list_all_grouped(conn)?;
    let mut backup_projects: Vec<BackupProject> = projects::list_all(conn)?
        .into_iter()
        .filter_map(|p| {
            let tag_names: Vec<String> = grouped
                .remove(&p.id)
                .unwrap_or_default()
                .into_iter()
                .map(|t| t.name)
                .collect();
            let has_something = p.pinned || p.notes.is_some() || !tag_names.is_empty();
            has_something.then_some(BackupProject {
                path: p.path,
                pinned: p.pinned,
                notes: p.notes,
                tags: tag_names,
            })
        })
        .collect();
    backup_projects.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(BackupWork {
        tags: backup_tags,
        projects: backup_projects,
    })
}

/// Serializa la copia como JSON.
pub fn to_json(backup: &Backup) -> Result<String> {
    serde_json::to_string_pretty(backup)
        .map_err(|err| AppError::Internal(format!("no se pudo serializar la copia: {err}")))
}

/// Lee y valida una copia; si algo no cuadra, la rechaza entera.
pub fn parse_backup(text: &str) -> Result<Backup> {
    let ajeno = || AppError::Validation("el fichero no es una copia de Mosaic".into());

    let value: serde_json::Value = serde_json::from_str(text).map_err(|_| ajeno())?;
    if value.get("format").and_then(|v| v.as_str()) != Some(FORMAT_TAG) {
        return Err(ajeno());
    }
    let version = value
        .get("version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| AppError::Validation("la copia no dice su versión".into()))?;
    if version > u64::from(FORMAT_VERSION) {
        return Err(AppError::Validation(format!(
            "la copia es de una versión más nueva de Mosaic (formato {version}); \
             actualiza Mosaic para importarla"
        )));
    }

    let backup: Backup = serde_json::from_value(value)
        .map_err(|err| AppError::Validation(format!("la copia está dañada: {err}")))?;
    validate(&backup)?;
    Ok(backup)
}

fn validate(backup: &Backup) -> Result<()> {
    let dañada = |motivo: String| AppError::Validation(format!("la copia está dañada: {motivo}"));

    validate_advanced(backup.settings.advanced.clone()).map_err(|e| dañada(e.to_string()))?;

    for p in &backup.scan_paths {
        if p.path.trim().is_empty() {
            return Err(dañada("hay una ruta de escaneo vacía".into()));
        }
    }

    let Some(work) = &backup.work else {
        return Ok(());
    };

    let mut nombres = BTreeSet::new();
    for t in &work.tags {
        tag::validate_name(&t.name).map_err(|e| dañada(e.to_string()))?;
        tag::normalize_color(&t.color).map_err(|e| dañada(e.to_string()))?;
        if !nombres.insert(tag::name_key(&t.name)) {
            return Err(dañada(format!("la etiqueta «{}» está repetida", t.name)));
        }
    }
    for p in &work.projects {
        if let Some(notes) = &p.notes {
            normalize_notes(notes).map_err(|e| dañada(e.to_string()))?;
        }
        for nombre in &p.tags {
            if !nombres.contains(&tag::name_key(nombre)) {
                return Err(dañada(format!(
                    "el proyecto {} lleva la etiqueta «{nombre}», que no está en la copia",
                    p.path
                )));
            }
        }
    }
    Ok(())
}

/// Calcula lo que haría importar, sin escribir.
pub fn plan_import(conn: &Connection, backup: &Backup) -> Result<ImportPlan> {
    reconcile(conn, backup, None)
}

/// Importa la copia en una sola transacción.
pub fn apply_import(conn: &mut Connection, backup: &Backup, now: i64) -> Result<ImportPlan> {
    let tx = conn.transaction()?;
    let plan = reconcile(&tx, backup, Some(now))?;
    tx.commit()?;
    Ok(plan)
}

// Previsualizar y aplicar pasan por aquí para que el resumen sea lo que se hace.
fn reconcile(conn: &Connection, backup: &Backup, write: Option<i64>) -> Result<ImportPlan> {
    let mut plan = ImportPlan {
        exported_at: backup.exported_at,
        app_version: backup.app_version.clone(),
        has_work: backup.work.is_some(),
        ..ImportPlan::default()
    };

    reconcile_settings(conn, &backup.settings, write.is_some(), &mut plan)?;

    for p in &backup.scan_paths {
        if scan_paths::find_by_path(conn, &p.path)?.is_some() {
            continue;
        }
        if !Path::new(&p.path).is_dir() {
            plan.scan_paths_missing.push(p.path.clone());
            continue;
        }
        plan.scan_paths_added += 1;
        if let Some(now) = write {
            let added = scan_paths::add(conn, &p.path, now)?;
            if !p.enabled {
                scan_paths::set_enabled(conn, added.id, false)?;
            }
        }
    }

    if let Some(work) = &backup.work {
        reconcile_work(conn, work, write, &mut plan)?;
    }
    Ok(plan)
}

fn reconcile_settings(
    conn: &Connection,
    incoming: &BackupSettings,
    write: bool,
    plan: &mut ImportPlan,
) -> Result<()> {
    let current = settings::load(conn)?;
    let here = AdvancedSettings::from(&current);
    let there = &incoming.advanced;

    let mut excluded_dirs = here.excluded_dirs.clone();
    for nombre in &there.excluded_dirs {
        if !excluded_dirs.contains(nombre) {
            excluded_dirs.push(nombre.clone());
        }
    }
    let merged = validate_advanced(AdvancedSettings {
        excluded_dirs,
        ..there.clone()
    })?;

    let mut changed = Vec::new();
    if merged.max_depth != here.max_depth {
        changed.push("max_depth");
    }
    if merged.excluded_dirs != here.excluded_dirs {
        changed.push("excluded_dirs");
    }
    if merged.max_entries_per_scan != here.max_entries_per_scan {
        changed.push("max_entries_per_scan");
    }
    if merged.scan_on_startup != here.scan_on_startup {
        changed.push("scan_on_startup");
    }
    if merged.git_refresh_interval_minutes != here.git_refresh_interval_minutes {
        changed.push("git_refresh_interval_minutes");
        plan.git_interval_changed = true;
    }
    // Vacío significa «el primero que haya», no una elección: no pisa la de aquí.
    let ide = !incoming.preferred_ide.is_empty() && incoming.preferred_ide != current.preferred_ide;
    let terminal = !incoming.preferred_terminal.is_empty()
        && incoming.preferred_terminal != current.preferred_terminal;
    if ide {
        changed.push("preferred_ide");
    }
    if terminal {
        changed.push("preferred_terminal");
    }
    plan.settings_changed = changed.into_iter().map(String::from).collect();

    if write {
        if merged != here {
            settings::save_advanced(conn, &merged)?;
        }
        if ide {
            settings::set_raw(conn, KEY_PREFERRED_IDE, &incoming.preferred_ide)?;
        }
        if terminal {
            settings::set_raw(conn, KEY_PREFERRED_TERMINAL, &incoming.preferred_terminal)?;
        }
    }
    Ok(())
}

fn reconcile_work(
    conn: &Connection,
    work: &BackupWork,
    write: Option<i64>,
    plan: &mut ImportPlan,
) -> Result<()> {
    // `None`: etiqueta aún sin crear en la previsualización; sus asignaciones cuentan igual.
    let mut tag_ids: HashMap<String, Option<i64>> = HashMap::new();
    for t in &work.tags {
        let key = tag::name_key(&t.name);
        let id = match tags::get_by_name_ci(conn, &t.name)? {
            Some(existing) => {
                plan.tags_existing += 1;
                Some(existing.id)
            }
            None => {
                plan.tags_added += 1;
                match write {
                    Some(now) => Some(tags::create(conn, &t.name, &t.color, now)?.id),
                    None => None,
                }
            }
        };
        tag_ids.insert(key, id);
    }

    let mut assigned = project_tags::list_all_grouped(conn)?;
    for p in &work.projects {
        let Some(local) = projects::find_by_path(conn, &p.path)? else {
            plan.projects_not_found += 1;
            continue;
        };
        plan.projects_matched += 1;

        if p.pinned && !local.pinned {
            plan.pins_added += 1;
            if write.is_some() {
                projects::set_pinned(conn, local.id, true)?;
            }
        }

        let local_tags = assigned.remove(&local.id).unwrap_or_default();
        for nombre in &p.tags {
            let id = tag_ids.get(&tag::name_key(nombre)).copied().flatten();
            let already = id.is_some_and(|id| local_tags.iter().any(|t| t.id == id));
            if already {
                continue;
            }
            plan.assignments_added += 1;
            if let Some(id) = id.filter(|_| write.is_some()) {
                project_tags::assign(conn, local.id, id)?;
            }
        }

        if let Some(theirs) = normalize_notes(p.notes.as_deref().unwrap_or(""))? {
            match local.notes.as_deref() {
                None => {
                    plan.notes_added += 1;
                    if let Some(now) = write {
                        projects::set_notes(conn, local.id, Some(&theirs), now)?;
                    }
                }
                Some(mine) if mine != theirs => plan.notes_conflicts.push(local.name.clone()),
                Some(_) => {}
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::project::DiscoveredProject;
    use crate::db::Db;

    fn db() -> Db {
        Db::open_in_memory().unwrap()
    }

    fn proyecto(conn: &Connection, path: &str) -> i64 {
        projects::upsert_by_path(
            conn,
            &DiscoveredProject {
                name: Path::new(path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                path: path.into(),
                is_git_repo: false,
                primary_language: None,
                markers: vec![],
            },
            1,
        )
        .unwrap()
        .id
    }

    fn sembrar(conn: &Connection, raiz: &str) {
        scan_paths::add(conn, raiz, 1).unwrap();
        let a = proyecto(conn, "/p/api");
        proyecto(conn, "/p/vacio");
        let cliente = tags::create(conn, "Cliente", "#3b82f6", 1).unwrap();
        tags::create(conn, "urgente", "#EF4444", 1).unwrap();
        project_tags::assign(conn, a, cliente.id).unwrap();
        projects::set_pinned(conn, a, true).unwrap();
        projects::set_notes(conn, a, Some("revisar facturas"), 2).unwrap();
        settings::set_raw(conn, KEY_PREFERRED_IDE, "code").unwrap();
    }

    fn exportar(db: &Db, include_work: bool) -> Backup {
        db.with_conn(|c| build_backup(c, include_work, 100, "0.1.0"))
            .unwrap()
    }

    #[test]
    fn export_keeps_only_projects_with_something_to_save() {
        let db = db();
        let raiz = tempfile::tempdir().unwrap();
        db.with_conn(|c| {
            sembrar(c, raiz.path().to_str().unwrap());
            Ok(())
        })
        .unwrap();

        let backup = exportar(&db, true);
        let work = backup.work.unwrap();
        assert_eq!(work.tags.len(), 2);
        assert_eq!(
            work.projects,
            vec![BackupProject {
                path: "/p/api".into(),
                pinned: true,
                notes: Some("revisar facturas".into()),
                tags: vec!["Cliente".into()],
            }]
        );
        assert_eq!(backup.settings.preferred_ide, "code");
        assert_eq!(backup.scan_paths.len(), 1);
    }

    #[test]
    fn export_without_work_carries_no_tags_or_notes() {
        let db = db();
        db.with_conn(|c| {
            sembrar(c, "/r");
            Ok(())
        })
        .unwrap();

        let backup = exportar(&db, false);
        assert!(backup.work.is_none());
        let json = to_json(&backup).unwrap();
        assert!(!json.contains("revisar facturas"));
        assert!(!json.contains("Cliente"));
    }

    #[test]
    fn round_trip_into_an_empty_database_restores_everything() {
        let origen = db();
        let raiz = tempfile::tempdir().unwrap();
        let raiz_str = raiz.path().to_str().unwrap().to_string();
        origen
            .with_conn(|c| {
                sembrar(c, &raiz_str);
                Ok(())
            })
            .unwrap();
        let texto = to_json(&exportar(&origen, true)).unwrap();

        let destino = db();
        let id = destino.with_conn(|c| Ok(proyecto(c, "/p/api"))).unwrap();
        let backup = parse_backup(&texto).unwrap();

        let previsto = destino.with_conn(|c| plan_import(c, &backup)).unwrap();
        let hecho = destino
            .with_conn_mut(|c| apply_import(c, &backup, 200))
            .unwrap();
        assert_eq!(previsto, hecho, "el resumen previo es lo que se hace");

        assert_eq!(hecho.scan_paths_added, 1);
        assert_eq!(hecho.tags_added, 2);
        assert_eq!(hecho.projects_matched, 1);
        assert_eq!(hecho.pins_added, 1);
        assert_eq!(hecho.assignments_added, 1);
        assert_eq!(hecho.notes_added, 1);
        assert_eq!(hecho.settings_changed, vec!["preferred_ide".to_string()]);

        destino
            .with_conn(|c| {
                let p = projects::get_by_id(c, id)?;
                assert!(p.pinned);
                assert_eq!(p.notes.as_deref(), Some("revisar facturas"));
                let etiquetas = project_tags::list_tags_for_project(c, id)?;
                assert_eq!(etiquetas[0].name, "Cliente");
                assert_eq!(etiquetas[0].color, "#3B82F6");
                assert_eq!(scan_paths::list(c)?[0].path, raiz_str);
                assert_eq!(settings::load(c)?.preferred_ide, "code");
                Ok(())
            })
            .unwrap();

        let otra_vez = destino.with_conn(|c| plan_import(c, &backup)).unwrap();
        assert!(otra_vez.is_empty(), "importar dos veces no hace nada más");
    }

    #[test]
    fn import_adds_and_never_deletes() {
        let db = db();
        let (id, mia) = db
            .with_conn(|c| {
                let id = proyecto(c, "/p/api");
                let mia = tags::create(c, "mia", "#22C55E", 1)?;
                project_tags::assign(c, id, mia.id)?;
                projects::set_pinned(c, id, true)?;
                projects::set_notes(c, id, Some("las mías"), 1)?;
                scan_paths::add(c, "/solo-aqui", 1)?;
                Ok((id, mia))
            })
            .unwrap();

        let backup = Backup {
            settings: BackupSettings {
                advanced: AdvancedSettings {
                    excluded_dirs: vec!["otra".into()],
                    ..AdvancedSettings::from(&settings::Settings::default())
                },
                preferred_ide: String::new(),
                preferred_terminal: String::new(),
            },
            scan_paths: vec![],
            work: Some(BackupWork {
                tags: vec![BackupTag {
                    name: "MIA".into(),
                    color: "#000000".into(),
                }],
                projects: vec![BackupProject {
                    path: "/p/api".into(),
                    pinned: false,
                    notes: Some("las suyas".into()),
                    tags: vec![],
                }],
            }),
            ..exportar(&db, false)
        };

        let plan = db.with_conn_mut(|c| apply_import(c, &backup, 5)).unwrap();
        assert_eq!(plan.tags_existing, 1);
        assert_eq!(plan.tags_added, 0);
        assert_eq!(plan.pins_added, 0);
        assert_eq!(plan.notes_conflicts, vec!["api".to_string()]);
        assert_eq!(plan.settings_changed, vec!["excluded_dirs".to_string()]);

        db.with_conn(|c| {
            let p = projects::get_by_id(c, id)?;
            assert!(p.pinned, "no desfija");
            assert_eq!(p.notes.as_deref(), Some("las mías"), "no pisa notas");
            assert_eq!(
                project_tags::list_tags_for_project(c, id)?,
                vec![mia.clone()]
            );
            assert_eq!(
                tags::get_by_id(c, mia.id)?.color,
                "#22C55E",
                "no cambia el color"
            );
            assert_eq!(scan_paths::list(c)?.len(), 1, "no quita rutas");
            let excluidas = settings::load(c)?.excluded_dirs;
            assert!(excluidas.contains(&"node_modules".to_string()));
            assert!(excluidas.contains(&"otra".to_string()));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn preview_writes_nothing() {
        let origen = db();
        let raiz = tempfile::tempdir().unwrap();
        origen
            .with_conn(|c| {
                sembrar(c, raiz.path().to_str().unwrap());
                Ok(())
            })
            .unwrap();
        let backup = exportar(&origen, true);

        let destino = db();
        destino.with_conn(|c| Ok(proyecto(c, "/p/api"))).unwrap();
        let plan = destino.with_conn(|c| plan_import(c, &backup)).unwrap();
        assert_eq!(plan.tags_added, 2);
        assert_eq!(
            plan.assignments_added, 1,
            "cuenta la de una etiqueta por crear"
        );

        destino
            .with_conn(|c| {
                assert!(tags::list_all(c)?.is_empty());
                assert!(scan_paths::list(c)?.is_empty());
                assert_eq!(settings::load(c)?.preferred_ide, "");
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn scan_paths_missing_on_this_machine_are_reported_not_added() {
        let origen = db();
        origen
            .with_conn(|c| {
                scan_paths::add(c, "/no/existe/en/este/equipo", 1)?;
                Ok(())
            })
            .unwrap();
        let backup = exportar(&origen, false);

        let destino = db();
        let plan = destino
            .with_conn_mut(|c| apply_import(c, &backup, 1))
            .unwrap();
        assert_eq!(plan.scan_paths_added, 0);
        assert_eq!(plan.scan_paths_missing, vec!["/no/existe/en/este/equipo"]);
        assert!(destino.with_conn(scan_paths::list).unwrap().is_empty());
    }

    #[test]
    fn projects_not_on_this_machine_are_counted_and_skipped() {
        let origen = db();
        origen
            .with_conn(|c| {
                let id = proyecto(c, "/solo/alli");
                projects::set_pinned(c, id, true)
            })
            .unwrap();
        let backup = exportar(&origen, true);

        let plan = db().with_conn(|c| plan_import(c, &backup)).unwrap();
        assert_eq!(plan.projects_not_found, 1);
        assert_eq!(plan.projects_matched, 0);
        assert_eq!(plan.pins_added, 0);
    }

    #[test]
    fn too_many_excluded_dirs_after_merging_is_rejected_before_writing() {
        let db = db();
        let mut backup = exportar(&db, false);
        backup.work = Some(BackupWork {
            tags: vec![BackupTag {
                name: "nueva".into(),
                color: "#000000".into(),
            }],
            projects: vec![],
        });
        backup.settings.advanced.excluded_dirs = (0..100).map(|i| format!("d{i}")).collect();

        assert!(db.with_conn_mut(|c| apply_import(c, &backup, 1)).is_err());
        assert!(db.with_conn(tags::list_all).unwrap().is_empty());
    }

    #[test]
    fn a_failure_halfway_rolls_back_what_was_already_written() {
        let origen = db();
        let raiz = tempfile::tempdir().unwrap();
        origen
            .with_conn(|c| {
                sembrar(c, raiz.path().to_str().unwrap());
                Ok(())
            })
            .unwrap();
        let backup = exportar(&origen, true);

        // Falla en la primera asignación, después de haber escrito etiquetas y ajustes.
        let destino = db();
        destino
            .with_conn(|c| {
                proyecto(c, "/p/api");
                c.execute_batch(
                    "CREATE TRIGGER romper BEFORE INSERT ON project_tags
                     BEGIN SELECT RAISE(ABORT, 'roto a propósito'); END;",
                )?;
                Ok(())
            })
            .unwrap();

        assert!(destino
            .with_conn_mut(|c| apply_import(c, &backup, 1))
            .is_err());
        destino
            .with_conn(|c| {
                assert!(tags::list_all(c)?.is_empty());
                assert!(scan_paths::list(c)?.is_empty());
                assert_eq!(settings::load(c)?.preferred_ide, "");
                assert!(!projects::find_by_path(c, "/p/api")?.unwrap().pinned);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn rejects_files_that_are_not_mosaic_backups() {
        for texto in ["", "hola", "[]", r#"{"format":"otra-cosa","version":1}"#] {
            let err = parse_backup(texto).unwrap_err().to_string();
            assert_eq!(err, "el fichero no es una copia de Mosaic", "{texto:?}");
        }
    }

    #[test]
    fn rejects_a_newer_format_with_a_clear_message() {
        let db = db();
        let mut value = serde_json::to_value(exportar(&db, false)).unwrap();
        value["version"] = (FORMAT_VERSION + 1).into();
        let err = parse_backup(&value.to_string()).unwrap_err().to_string();
        assert!(err.contains("versión más nueva"), "{err}");
    }

    #[test]
    fn rejects_damaged_content() {
        let db = db();
        let base = exportar(&db, true);

        let mut sin_campo = serde_json::to_value(&base).unwrap();
        sin_campo["settings"]
            .as_object_mut()
            .unwrap()
            .remove("max_depth");

        let mut profundidad = base.clone();
        profundidad.settings.advanced.max_depth = 99;

        let mut color = base.clone();
        color.work = Some(BackupWork {
            tags: vec![BackupTag {
                name: "x".into(),
                color: "rojo".into(),
            }],
            projects: vec![],
        });

        let mut huerfana = base.clone();
        huerfana.work = Some(BackupWork {
            tags: vec![],
            projects: vec![BackupProject {
                path: "/p".into(),
                pinned: false,
                notes: None,
                tags: vec!["fantasma".into()],
            }],
        });

        let mut repetida = base.clone();
        repetida.work = Some(BackupWork {
            tags: vec![
                BackupTag {
                    name: "Uno".into(),
                    color: "#000000".into(),
                },
                BackupTag {
                    name: "uno".into(),
                    color: "#FFFFFF".into(),
                },
            ],
            projects: vec![],
        });

        let textos = [
            sin_campo.to_string(),
            to_json(&profundidad).unwrap(),
            to_json(&color).unwrap(),
            to_json(&huerfana).unwrap(),
            to_json(&repetida).unwrap(),
        ];
        for texto in textos {
            let err = parse_backup(&texto).unwrap_err().to_string();
            assert!(err.starts_with("la copia está dañada"), "{err}");
        }
    }
}
