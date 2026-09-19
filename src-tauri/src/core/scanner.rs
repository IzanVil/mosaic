//! Descubrimiento de proyectos en disco.
//!
//! # Qué cuenta como proyecto
//!
//! Una carpeta es candidata si contiene al menos uno de [`PROJECT_MARKERS`].
//! Una candidata se registra como proyecto cuando:
//!
//! 1. contiene su propio `.git`, aunque un ancestro ya esté registrado; **o**
//! 2. ningún ancestro suyo está registrado ya como proyecto.
//!
//! Así, un monorepo (`.git` en la raíz y `package.json` en `packages/*`) produce
//! una sola tarjeta, mientras que dos repositorios anidados (`.git` en la raíz y
//! en `sub/`) producen dos.
//!
//! # Recorrido
//!
//! Se usa `walkdir` con `follow_links(false)`: los enlaces simbólicos no se
//! siguen, lo que hace inmune al escáner frente a ciclos de symlinks. La
//! profundidad se cuenta con la raíz como nivel 0, y el recorrido se aborta con
//! un aviso si supera `max_entries_per_scan` entradas.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

use crate::config::settings::Settings;
use crate::core::project::DiscoveredProject;

/// Ficheros y carpetas cuya presencia delata un proyecto.
pub const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "composer.json",
    "Gemfile",
];

/// Parámetros de un escaneo, derivados de [`Settings`].
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Profundidad máxima, con la raíz como nivel 0.
    pub max_depth: usize,
    /// Nombres de directorio que no se recorren.
    pub excluded_dirs: HashSet<String>,
    /// Tope de entradas visitadas antes de abortar el recorrido.
    pub max_entries: usize,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self::from_settings(&Settings::default())
    }
}

impl ScanOptions {
    /// Construye las opciones de escaneo a partir de los ajustes persistidos.
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            max_depth: settings.max_depth,
            excluded_dirs: settings.excluded_dirs.iter().cloned().collect(),
            max_entries: settings.max_entries_per_scan,
        }
    }
}

/// Resultado del recorrido de una raíz.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanOutcome {
    pub projects: Vec<DiscoveredProject>,
    /// Entradas del sistema de ficheros visitadas.
    pub entries_visited: usize,
    /// `true` si se alcanzó el tope de entradas y el recorrido quedó incompleto.
    pub truncated: bool,
}

/// Candidata detectada durante el recorrido, antes de aplicar la regla de anidamiento.
struct Candidate {
    path: PathBuf,
    depth: usize,
    markers: Vec<String>,
}

/// Indica si una entrada es un directorio excluido.
///
/// La raíz nunca se excluye: si el usuario configura `~/code/node_modules`
/// como ruta de escaneo, es su decisión.
fn is_excluded(entry: &DirEntry, excluded: &HashSet<String>) -> bool {
    entry.depth() > 0
        && entry.file_type().is_dir()
        && entry
            .file_name()
            .to_str()
            .is_some_and(|name| excluded.contains(name))
}

/// Devuelve los marcadores presentes en `dir`, en el orden de [`PROJECT_MARKERS`].
fn detect_markers(dir: &Path) -> Vec<String> {
    PROJECT_MARKERS
        .iter()
        .filter(|marker| dir.join(marker).exists())
        .map(|marker| (*marker).to_string())
        .collect()
}

/// Recorre `root` y devuelve los proyectos encontrados.
pub fn discover(root: &Path, options: &ScanOptions) -> ScanOutcome {
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut entries_visited = 0usize;
    let mut truncated = false;

    let walker = WalkDir::new(root)
        .max_depth(options.max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !is_excluded(entry, &options.excluded_dirs));

    for entry in walker {
        entries_visited += 1;
        if entries_visited > options.max_entries {
            tracing::warn!(
                root = %root.display(),
                max_entries = options.max_entries,
                "escaneo abortado: se superó el tope de entradas"
            );
            truncated = true;
            break;
        }

        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                tracing::warn!(%err, "entrada ilegible, se omite");
                continue;
            }
        };

        // `is_dir()` es falso para enlaces simbólicos con `follow_links(false)`,
        // así que un ciclo de symlinks nunca se recorre.
        if !entry.file_type().is_dir() {
            continue;
        }

        let markers = detect_markers(entry.path());
        if markers.is_empty() {
            continue;
        }

        candidates.push(Candidate {
            path: entry.path().to_path_buf(),
            depth: entry.depth(),
            markers,
        });
    }

    let projects = register_candidates(candidates, options);

    tracing::info!(
        root = %root.display(),
        entries = entries_visited,
        projects = projects.len(),
        truncated,
        "raíz escaneada"
    );

    ScanOutcome {
        projects,
        entries_visited,
        truncated,
    }
}

/// Aplica la regla de anidamiento a las candidatas y calcula su lenguaje primario.
fn register_candidates(
    mut candidates: Vec<Candidate>,
    options: &ScanOptions,
) -> Vec<DiscoveredProject> {
    // De menos a más profundo: todo ancestro se decide antes que sus descendientes.
    candidates.sort_by(|a, b| a.depth.cmp(&b.depth).then_with(|| a.path.cmp(&b.path)));

    let mut registered: Vec<PathBuf> = Vec::new();
    let mut projects = Vec::new();

    for candidate in candidates {
        let has_own_git = candidate.markers.iter().any(|marker| marker == ".git");
        let inside_registered_project = registered
            .iter()
            .any(|root| candidate.path.starts_with(root) && candidate.path != *root);

        if !has_own_git && inside_registered_project {
            continue;
        }

        projects.push(DiscoveredProject {
            name: DiscoveredProject::name_from_path(&candidate.path),
            is_git_repo: has_own_git,
            primary_language: detect_primary_language(&candidate.path, &options.excluded_dirs),
            markers: candidate.markers,
            path: candidate.path.clone(),
        });
        registered.push(candidate.path);
    }

    projects
}

/// Profundidad a la que se cuentan extensiones dentro de un proyecto.
const LANGUAGE_SCAN_DEPTH: usize = 2;

/// Extensión de fichero -> lenguaje. Se omiten deliberadamente Markdown, JSON,
/// YAML y TOML: aparecen en casi todos los repositorios y no dicen nada sobre
/// el lenguaje principal.
const EXTENSION_LANGUAGES: &[(&str, &str)] = &[
    ("rs", "Rust"),
    ("ts", "TypeScript"),
    ("tsx", "TypeScript"),
    ("js", "JavaScript"),
    ("jsx", "JavaScript"),
    ("mjs", "JavaScript"),
    ("cjs", "JavaScript"),
    ("svelte", "Svelte"),
    ("vue", "Vue"),
    ("py", "Python"),
    ("go", "Go"),
    ("java", "Java"),
    ("kt", "Kotlin"),
    ("kts", "Kotlin"),
    ("rb", "Ruby"),
    ("php", "PHP"),
    ("cs", "C#"),
    ("swift", "Swift"),
    ("c", "C"),
    ("h", "C"),
    ("cpp", "C++"),
    ("cc", "C++"),
    ("cxx", "C++"),
    ("hpp", "C++"),
    ("zig", "Zig"),
    ("lua", "Lua"),
    ("dart", "Dart"),
    ("ex", "Elixir"),
    ("exs", "Elixir"),
    ("sh", "Shell"),
    ("bash", "Shell"),
    ("html", "HTML"),
    ("css", "CSS"),
    ("scss", "CSS"),
];

/// Lenguajes que puede designar un `package.json`, a resolver contando extensiones.
const JS_FAMILY: &[&str] = &["TypeScript", "JavaScript", "Svelte", "Vue"];

/// Lenguaje que implica inequívocamente cada fichero de manifiesto.
///
/// `package.json` no aparece aquí: no distingue TypeScript de JavaScript, de
/// Svelte o de Vue, así que se resuelve contando extensiones.
fn language_from_manifest(dir: &Path) -> Option<&'static str> {
    const MANIFESTS: &[(&str, &str)] = &[
        ("Cargo.toml", "Rust"),
        ("go.mod", "Go"),
        ("pyproject.toml", "Python"),
        ("build.gradle.kts", "Kotlin"),
        ("build.gradle", "Java"),
        ("pom.xml", "Java"),
        ("composer.json", "PHP"),
        ("Gemfile", "Ruby"),
    ];

    MANIFESTS
        .iter()
        .find(|(file, _)| dir.join(file).exists())
        .map(|(_, language)| *language)
}

/// Cuenta ficheros por lenguaje hasta [`LANGUAGE_SCAN_DEPTH`] niveles.
fn count_extensions(dir: &Path, excluded: &HashSet<String>) -> HashMap<&'static str, usize> {
    let mut counts: HashMap<&'static str, usize> = HashMap::new();

    let walker = WalkDir::new(dir)
        .max_depth(LANGUAGE_SCAN_DEPTH)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !is_excluded(entry, excluded));

    for entry in walker.flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(extension) = entry.path().extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if let Some((_, language)) = EXTENSION_LANGUAGES
            .iter()
            .find(|(candidate, _)| *candidate == extension)
        {
            *counts.entry(*language).or_insert(0) += 1;
        }
    }

    counts
}

/// Devuelve el lenguaje más frecuente entre `allowed`, o el más frecuente de
/// todos si `allowed` es `None`. Los empates se rompen por orden alfabético
/// para que el resultado sea determinista.
fn top_language(
    counts: &HashMap<&'static str, usize>,
    allowed: Option<&[&str]>,
) -> Option<&'static str> {
    counts
        .iter()
        .filter(|(language, _)| allowed.is_none_or(|list| list.contains(language)))
        .max_by(|(lang_a, count_a), (lang_b, count_b)| {
            count_a.cmp(count_b).then_with(|| lang_b.cmp(lang_a))
        })
        .map(|(language, _)| *language)
}

/// Deduce el lenguaje principal de un proyecto.
///
/// Heurística, en orden:
///
/// 1. **Manifiesto.** `Cargo.toml` -> Rust, `go.mod` -> Go, `pyproject.toml` ->
///    Python, `build.gradle.kts` -> Kotlin, `build.gradle`/`pom.xml` -> Java,
///    `composer.json` -> PHP, `Gemfile` -> Ruby.
/// 2. **`package.json`.** Ambiguo por sí solo: se resuelve contando extensiones
///    y quedándose con el lenguaje más frecuente de la familia JS; si no hay
///    ninguno, JavaScript.
/// 3. **Conteo de extensiones** hasta dos niveles, ignorando los directorios
///    excluidos y los formatos de documentación y configuración.
/// 4. Si nada de lo anterior decide, `None`.
pub fn detect_primary_language(dir: &Path, excluded: &HashSet<String>) -> Option<String> {
    if let Some(language) = language_from_manifest(dir) {
        return Some(language.to_string());
    }

    let counts = count_extensions(dir, excluded);

    if dir.join("package.json").exists() {
        return Some(
            top_language(&counts, Some(JS_FAMILY))
                .unwrap_or("JavaScript")
                .to_string(),
        );
    }

    top_language(&counts, None).map(str::to_string)
}

/// Resumen de un escaneo completo, tal y como lo recibe el frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanSummary {
    /// Raíces habilitadas que se han recorrido de verdad.
    pub roots_scanned: usize,
    /// Raíces configuradas que ya no existen en disco.
    pub roots_unavailable: usize,
    pub entries_visited: usize,
    pub projects_found: usize,
    pub projects_new: usize,
    /// Proyectos ya conocidos cuyos metadatos han cambiado.
    pub projects_updated: usize,
    /// Proyectos que han dejado de verse y se han marcado como ausentes.
    pub projects_missing: usize,
    /// `true` si alguna raíz alcanzó el tope de entradas.
    pub truncated: bool,
    pub elapsed_ms: u64,
}

/// Escanea todas las rutas habilitadas y sincroniza la tabla `projects`.
///
/// Los proyectos que dejan de verse se marcan como ausentes pero nunca se
/// borran, para no perder las etiquetas y notas que el usuario les haya puesto.
/// Escanea al arrancar, si el usuario lo tiene activado.
///
/// Devuelve `None` cuando el ajuste `scan.on_startup` está desactivado, que es
/// el valor por defecto: quien no lo pidió no paga el recorrido del disco cada
/// vez que abre la aplicación. La decisión vive aquí, y no en el arranque de
/// Tauri, para que se pueda comprobar con un test.
pub fn run_startup_scan(db: &crate::db::Db) -> crate::errors::Result<Option<ScanSummary>> {
    let settings = db.with_conn(crate::config::settings::load)?;
    if !settings.scan_on_startup {
        tracing::debug!("escaneo al arrancar desactivado");
        return Ok(None);
    }

    tracing::info!("escaneo al arrancar activado");
    run_full_scan(db).map(Some)
}

pub fn run_full_scan(db: &crate::db::Db) -> crate::errors::Result<ScanSummary> {
    use crate::db::repositories::{projects as projects_repo, scan_paths as scan_paths_repo};

    let started = std::time::Instant::now();
    let now = crate::core::now_ts();

    let (settings, roots) = db.with_conn(|conn| {
        Ok((
            crate::config::settings::load(conn)?,
            scan_paths_repo::list_enabled(conn)?,
        ))
    })?;
    let options = ScanOptions::from_settings(&settings);

    let mut summary = ScanSummary {
        roots_scanned: 0,
        roots_unavailable: 0,
        entries_visited: 0,
        projects_found: 0,
        projects_new: 0,
        projects_updated: 0,
        projects_missing: 0,
        truncated: false,
        elapsed_ms: 0,
    };
    let mut scanned_roots: Vec<PathBuf> = Vec::new();
    let mut seen_ids: HashSet<i64> = HashSet::new();

    for root in &roots {
        let root_path = PathBuf::from(&root.path);
        if !root_path.is_dir() {
            tracing::warn!(path = %root.path, "ruta de escaneo no disponible, se omite");
            summary.roots_unavailable += 1;
            continue;
        }

        let outcome = discover(&root_path, &options);
        summary.roots_scanned += 1;
        summary.entries_visited += outcome.entries_visited;
        summary.projects_found += outcome.projects.len();
        summary.truncated |= outcome.truncated;

        db.with_conn(|conn| {
            for project in &outcome.projects {
                let upsert = projects_repo::upsert_by_path(conn, project, now)?;
                seen_ids.insert(upsert.id);
                if upsert.created {
                    summary.projects_new += 1;
                } else if upsert.metadata_changed {
                    summary.projects_updated += 1;
                }
            }
            scan_paths_repo::set_last_scan_at(conn, root.id, now)
        })?;

        scanned_roots.push(root_path);
    }

    summary.projects_missing = db.with_conn(|conn| {
        projects_repo::mark_missing_under_roots(conn, &scanned_roots, &seen_ids)
    })?;

    summary.elapsed_ms = started.elapsed().as_millis() as u64;
    tracing::info!(?summary, "escaneo completo");
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// Crea un directorio y todos sus padres.
    fn mkdir(path: &Path) {
        fs::create_dir_all(path).unwrap();
    }

    /// Crea un fichero vacío y todos sus directorios padre.
    fn touch(path: &Path) {
        mkdir(path.parent().unwrap());
        fs::write(path, "").unwrap();
    }

    /// Nombres de los proyectos encontrados, ordenados alfabéticamente.
    fn names(outcome: &ScanOutcome) -> Vec<String> {
        let mut names: Vec<String> = outcome
            .projects
            .iter()
            .map(|project| project.name.clone())
            .collect();
        names.sort();
        names
    }

    fn options() -> ScanOptions {
        ScanOptions::default()
    }

    #[test]
    fn detects_every_project_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        for marker in PROJECT_MARKERS {
            let project = root.join(format!("proyecto-{}", marker.trim_start_matches('.')));
            if *marker == ".git" {
                mkdir(&project.join(".git"));
            } else {
                touch(&project.join(marker));
            }
        }

        let outcome = discover(root, &options());

        assert_eq!(
            outcome.projects.len(),
            PROJECT_MARKERS.len(),
            "cada marcador debe delatar un proyecto: {:?}",
            names(&outcome)
        );
    }

    #[test]
    fn detects_git_repo_when_dot_git_is_a_file() {
        // Los worktrees y submódulos guardan un fichero `.git`, no un directorio.
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("worktree");
        touch(&project.join(".git"));

        let outcome = discover(tmp.path(), &options());

        assert_eq!(outcome.projects.len(), 1);
        assert!(outcome.projects[0].is_git_repo);
    }

    #[test]
    fn root_itself_can_be_a_project() {
        let tmp = tempfile::tempdir().unwrap();
        touch(&tmp.path().join("Cargo.toml"));

        let outcome = discover(tmp.path(), &options());

        assert_eq!(outcome.projects.len(), 1);
        assert_eq!(outcome.projects[0].path, tmp.path());
    }

    #[test]
    fn respects_max_depth_counting_root_as_zero() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(&root.join("n1/n2/n3/n4/Cargo.toml"));
        touch(&root.join("n1/n2/n3/n4/n5/Cargo.toml"));

        let outcome = discover(root, &options());

        assert_eq!(
            names(&outcome),
            vec!["n4".to_string()],
            "n4 está en el nivel 4; n5, en el 5, queda fuera"
        );
    }

    #[test]
    fn max_depth_zero_only_looks_at_the_root() {
        let tmp = tempfile::tempdir().unwrap();
        touch(&tmp.path().join("hijo/Cargo.toml"));

        let opts = ScanOptions {
            max_depth: 0,
            ..options()
        };
        let outcome = discover(tmp.path(), &opts);

        assert!(outcome.projects.is_empty());
    }

    #[test]
    fn skips_excluded_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(&root.join("app/package.json"));
        touch(&root.join("node_modules/paquete/package.json"));
        touch(&root.join("app/node_modules/otro/package.json"));
        touch(&root.join("target/debug/Cargo.toml"));
        touch(&root.join(".venv/lib/pyproject.toml"));

        let outcome = discover(root, &options());

        assert_eq!(names(&outcome), vec!["app".to_string()]);
    }

    #[test]
    fn excluded_directory_can_still_be_the_scan_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("node_modules");
        touch(&root.join("app/Cargo.toml"));

        let outcome = discover(&root, &options());

        assert_eq!(names(&outcome), vec!["app".to_string()]);
    }

    #[test]
    #[cfg(unix)]
    fn circular_symlink_does_not_hang_the_scan() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let app = root.join("app");
        mkdir(&app.join(".git"));
        // Ciclo: app/bucle apunta a su propio directorio padre.
        symlink(&app, app.join("bucle")).unwrap();
        symlink(root, root.join("raiz-otra-vez")).unwrap();

        let outcome = discover(root, &options());

        assert_eq!(
            names(&outcome),
            vec!["app".to_string()],
            "follow_links(false) impide recorrer el ciclo"
        );
    }

    #[test]
    fn does_not_descend_into_a_registered_project() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(&root.join("app/package.json"));
        touch(&root.join("app/sub/package.json"));

        let outcome = discover(root, &options());

        assert_eq!(names(&outcome), vec!["app".to_string()]);
    }

    #[test]
    fn fake_monorepo_yields_a_single_project() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mono = root.join("mono");
        mkdir(&mono.join(".git"));
        touch(&mono.join("package.json"));
        touch(&mono.join("packages/a/package.json"));
        touch(&mono.join("packages/b/package.json"));

        let outcome = discover(root, &options());

        assert_eq!(names(&outcome), vec!["mono".to_string()]);
    }

    #[test]
    fn nested_repositories_yield_two_projects() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let repo = root.join("repo");
        mkdir(&repo.join(".git"));
        mkdir(&repo.join("sub/.git"));
        touch(&repo.join("sub/package.json"));

        let outcome = discover(root, &options());

        assert_eq!(names(&outcome), vec!["repo".to_string(), "sub".to_string()]);
        assert!(outcome.projects.iter().all(|project| project.is_git_repo));
    }

    #[test]
    fn nested_repository_is_detected_even_two_levels_deep() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let repo = root.join("repo");
        mkdir(&repo.join(".git"));
        touch(&repo.join("packages/a/package.json"));
        mkdir(&repo.join("packages/b/.git"));

        let outcome = discover(root, &options());

        assert_eq!(names(&outcome), vec!["b".to_string(), "repo".to_string()]);
    }

    #[test]
    fn aborts_and_flags_truncation_over_the_entry_budget() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for i in 0..20 {
            touch(&root.join(format!("relleno-{i}.txt")));
        }

        let opts = ScanOptions {
            max_entries: 5,
            ..options()
        };
        let outcome = discover(root, &opts);

        assert!(outcome.truncated);
        assert!(outcome.entries_visited <= 6);
    }

    fn excluded() -> HashSet<String> {
        options().excluded_dirs
    }

    #[test]
    fn language_from_manifest_wins_over_file_counts() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        touch(&project.join("Cargo.toml"));
        // Ruido: más ficheros Python que Rust, pero el manifiesto manda.
        for i in 0..5 {
            touch(&project.join(format!("script-{i}.py")));
        }
        touch(&project.join("src/main.rs"));

        assert_eq!(
            detect_primary_language(project, &excluded()).as_deref(),
            Some("Rust")
        );
    }

    #[test]
    fn every_unambiguous_manifest_maps_to_its_language() {
        let cases = [
            ("Cargo.toml", "Rust"),
            ("go.mod", "Go"),
            ("pyproject.toml", "Python"),
            ("build.gradle.kts", "Kotlin"),
            ("build.gradle", "Java"),
            ("pom.xml", "Java"),
            ("composer.json", "PHP"),
            ("Gemfile", "Ruby"),
        ];

        for (manifest, language) in cases {
            let tmp = tempfile::tempdir().unwrap();
            touch(&tmp.path().join(manifest));

            assert_eq!(
                detect_primary_language(tmp.path(), &excluded()).as_deref(),
                Some(language),
                "{manifest} debería detectarse como {language}"
            );
        }
    }

    #[test]
    fn counts_extensions_when_there_is_no_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        mkdir(&project.join(".git"));
        touch(&project.join("a.py"));
        touch(&project.join("modulo/b.py"));
        touch(&project.join("c.rs"));

        assert_eq!(
            detect_primary_language(project, &excluded()).as_deref(),
            Some("Python")
        );
    }

    #[test]
    fn package_json_is_resolved_by_counting_js_family_extensions() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        touch(&project.join("package.json"));
        touch(&project.join("src/app.ts"));
        touch(&project.join("src/util.ts"));
        touch(&project.join("src/legacy.js"));

        assert_eq!(
            detect_primary_language(project, &excluded()).as_deref(),
            Some("TypeScript")
        );
    }

    #[test]
    fn package_json_without_sources_falls_back_to_javascript() {
        let tmp = tempfile::tempdir().unwrap();
        touch(&tmp.path().join("package.json"));

        assert_eq!(
            detect_primary_language(tmp.path(), &excluded()).as_deref(),
            Some("JavaScript")
        );
    }

    #[test]
    fn extension_counting_ignores_excluded_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        mkdir(&project.join(".git"));
        touch(&project.join("main.py"));
        for i in 0..10 {
            touch(&project.join(format!("node_modules/dep-{i}.js")));
        }

        assert_eq!(
            detect_primary_language(project, &excluded()).as_deref(),
            Some("Python")
        );
    }

    #[test]
    fn extension_counting_stops_at_two_levels() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        mkdir(&project.join(".git"));
        touch(&project.join("a.rs"));
        // Nivel 3: fuera del alcance del conteo, por muchos que haya.
        for i in 0..10 {
            touch(&project.join(format!("uno/dos/tres/f-{i}.py")));
        }

        assert_eq!(
            detect_primary_language(project, &excluded()).as_deref(),
            Some("Rust")
        );
    }

    #[test]
    fn documentation_and_config_files_do_not_decide_the_language() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        mkdir(&project.join(".git"));
        for i in 0..5 {
            touch(&project.join(format!("doc-{i}.md")));
            touch(&project.join(format!("conf-{i}.yaml")));
        }

        assert_eq!(detect_primary_language(project, &excluded()), None);
    }

    #[test]
    fn ties_are_broken_alphabetically_for_determinism() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        mkdir(&project.join(".git"));
        touch(&project.join("a.go"));
        touch(&project.join("b.rs"));

        assert_eq!(
            detect_primary_language(project, &excluded()).as_deref(),
            Some("Go"),
            "empate a uno: gana el primero alfabéticamente"
        );
    }

    #[test]
    fn project_without_recognisable_sources_has_no_language() {
        let tmp = tempfile::tempdir().unwrap();
        mkdir(&tmp.path().join(".git"));

        assert_eq!(detect_primary_language(tmp.path(), &excluded()), None);
    }

    #[test]
    fn reports_the_markers_that_matched() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("app");
        mkdir(&project.join(".git"));
        touch(&project.join("Cargo.toml"));

        let outcome = discover(tmp.path(), &options());

        assert_eq!(
            outcome.projects[0].markers,
            vec![".git".to_string(), "Cargo.toml".to_string()]
        );
    }
}
