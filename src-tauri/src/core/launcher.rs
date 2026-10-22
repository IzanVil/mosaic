//! Detección y lanzamiento de aplicaciones externas.
//!
//! Nunca se construye una línea de shell: la ruta va como argumento suelto de
//! `Command`, así que una carpeta con comillas, espacios o `;` no ejecuta nada.
//!
//! Los enlaces de un README son de terceros: solo se abren `http` y `https`. Un
//! `file:` abriría el disco, un `javascript:` ejecutaría código y un `vscode:`
//! lanzaría aplicaciones. Se validan con el crate `url`, que sigue el mismo
//! estándar que el navegador: lo que se valida es lo que se abre.

use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::db::repositories::{projects as projects_repo, settings as settings_repo};
use crate::db::Db;
use crate::errors::{AppError, Result};

/// Qué tipo de aplicación abre un proyecto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppKind {
    Ide,
    Terminal,
    FileManager,
}

/// Una aplicación encontrada en el sistema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedApp {
    /// Nombre del ejecutable; es el identificador que se guarda en los ajustes.
    pub id: String,
    /// Nombre para mostrar al usuario.
    pub name: String,
    pub kind: AppKind,
}

/// Candidata a aplicación: ejecutable, nombre visible y cómo se le pasa el directorio.
struct Candidate {
    bin: &'static str,
    name: &'static str,
    /// Argumento que precede a la ruta. `None` si la ruta va suelta.
    dir_flag: Option<&'static str>,
}

/// IDEs y editores reconocidos, por orden de preferencia cuando hay varios.
const IDES: &[Candidate] = &[
    Candidate {
        bin: "code",
        name: "VS Code",
        dir_flag: None,
    },
    Candidate {
        bin: "code-insiders",
        name: "VS Code Insiders",
        dir_flag: None,
    },
    Candidate {
        bin: "codium",
        name: "VSCodium",
        dir_flag: None,
    },
    Candidate {
        bin: "zed",
        name: "Zed",
        dir_flag: None,
    },
    Candidate {
        bin: "subl",
        name: "Sublime Text",
        dir_flag: None,
    },
    Candidate {
        bin: "idea",
        name: "IntelliJ IDEA",
        dir_flag: None,
    },
    Candidate {
        bin: "webstorm",
        name: "WebStorm",
        dir_flag: None,
    },
    Candidate {
        bin: "pycharm",
        name: "PyCharm",
        dir_flag: None,
    },
    Candidate {
        bin: "rustrover",
        name: "RustRover",
        dir_flag: None,
    },
    Candidate {
        bin: "goland",
        name: "GoLand",
        dir_flag: None,
    },
    Candidate {
        bin: "clion",
        name: "CLion",
        dir_flag: None,
    },
    Candidate {
        bin: "phpstorm",
        name: "PhpStorm",
        dir_flag: None,
    },
    Candidate {
        bin: "rubymine",
        name: "RubyMine",
        dir_flag: None,
    },
    Candidate {
        bin: "nvim",
        name: "Neovim",
        dir_flag: None,
    },
    Candidate {
        bin: "hx",
        name: "Helix",
        dir_flag: None,
    },
];

/// Emuladores de terminal reconocidos, por orden de preferencia.
const TERMINALS: &[Candidate] = &[
    Candidate {
        bin: "kitty",
        name: "kitty",
        dir_flag: Some("--directory"),
    },
    Candidate {
        bin: "alacritty",
        name: "Alacritty",
        dir_flag: Some("--working-directory"),
    },
    Candidate {
        bin: "wezterm",
        name: "WezTerm",
        dir_flag: Some("--cwd"),
    },
    Candidate {
        bin: "ghostty",
        name: "Ghostty",
        dir_flag: Some("--working-directory"),
    },
    Candidate {
        bin: "foot",
        name: "foot",
        dir_flag: Some("--working-directory"),
    },
    Candidate {
        bin: "konsole",
        name: "Konsole",
        dir_flag: Some("--workdir"),
    },
    Candidate {
        bin: "gnome-terminal",
        name: "GNOME Terminal",
        dir_flag: Some("--working-directory"),
    },
    Candidate {
        bin: "xfce4-terminal",
        name: "Xfce Terminal",
        dir_flag: Some("--working-directory"),
    },
    Candidate {
        bin: "tilix",
        name: "Tilix",
        dir_flag: Some("--working-directory"),
    },
    Candidate {
        bin: "wt.exe",
        name: "Windows Terminal",
        dir_flag: Some("-d"),
    },
];

fn candidates(kind: AppKind) -> &'static [Candidate] {
    match kind {
        AppKind::Ide => IDES,
        AppKind::Terminal => TERMINALS,
        // El explorador de archivos lo resuelve el sistema, no una lista nuestra.
        AppKind::FileManager => &[],
    }
}

/// Indica si `bin` existe y es ejecutable en algún directorio del `PATH`.
pub fn is_on_path(bin: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };

    std::env::split_paths(&path).any(|dir| {
        let candidate = dir.join(bin);
        candidate.is_file() && is_executable(&candidate)
    })
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    // En Windows basta con que el fichero exista con la extensión adecuada.
    true
}

/// Aplicaciones de un tipo disponibles en el sistema, en orden de preferencia.
pub fn detect(kind: AppKind) -> Vec<DetectedApp> {
    detect_with(kind, is_on_path)
}

/// Igual que [`detect`], con la resolución de ejecutables inyectada para tests.
fn detect_with(kind: AppKind, available: impl Fn(&str) -> bool) -> Vec<DetectedApp> {
    candidates(kind)
        .iter()
        .filter(|candidate| available(candidate.bin))
        .map(|candidate| DetectedApp {
            id: candidate.bin.to_string(),
            name: candidate.name.to_string(),
            kind,
        })
        .collect()
}

/// El ejecutable preferido si sigue instalado; si no, el primero que se detecte.
fn resolve(kind: AppKind, preferred: Option<&str>) -> Option<&'static Candidate> {
    resolve_with(kind, preferred, is_on_path)
}

/// Igual que [`resolve`], con la resolución de ejecutables inyectada para tests.
fn resolve_with(
    kind: AppKind,
    preferred: Option<&str>,
    available: impl Fn(&str) -> bool,
) -> Option<&'static Candidate> {
    let installed: Vec<&'static Candidate> = candidates(kind)
        .iter()
        .filter(|candidate| available(candidate.bin))
        .collect();

    if let Some(preferred) = preferred.filter(|id| !id.is_empty()) {
        if let Some(found) = installed
            .iter()
            .find(|candidate| candidate.bin == preferred)
        {
            return Some(found);
        }
        tracing::warn!(
            preferred,
            ?kind,
            "la aplicación preferida no está disponible, se usa la primera que haya"
        );
    }

    installed.first().copied()
}

/// Construye el comando que abre `path` con `candidate`.
fn build_command(candidate: &Candidate, path: &Path) -> Command {
    let mut command = Command::new(candidate.bin);
    match candidate.dir_flag {
        Some(flag) => {
            command.arg(flag).arg(path);
        }
        None => {
            command.arg(path);
        }
    }
    // Hacen falta la bandera y el directorio: los terminales que hablan con un
    // servidor ya abierto no lo heredan, y los que no tienen bandera solo lo heredan.
    command.current_dir(path);
    command
}

/// Abre `path` con la aplicación preferida de su tipo, o con la primera que haya.
pub fn open_path(kind: AppKind, path: &Path, preferred: Option<&str>) -> Result<()> {
    if !path.is_dir() {
        return Err(AppError::invalid_path(
            path,
            "path_missing",
            "ya no existe en el disco",
        ));
    }

    let none_installed = match kind {
        AppKind::FileManager => {
            open::that_detached(path)?;
            tracing::info!(path = %path.display(), "abierto en el explorador de archivos");
            return Ok(());
        }
        AppKind::Ide => {
            AppError::not_found("no_editor", "ningún IDE o editor reconocido en el PATH")
        }
        AppKind::Terminal => AppError::not_found(
            "no_terminal",
            "ningún emulador de terminal reconocido en el PATH",
        ),
    };
    let candidate = resolve(kind, preferred).ok_or(none_installed)?;

    build_command(candidate, path).spawn().map_err(|err| {
        AppError::internal(
            "launch_failed",
            format!("no se pudo lanzar {}: {err}", candidate.bin),
        )
        .with("app", candidate.bin)
        .with("detail", err.to_string())
    })?;

    tracing::info!(app = candidate.bin, path = %path.display(), "proyecto abierto");
    Ok(())
}

/// Abre un proyecto y apunta cuándo, solo si el lanzamiento ha salido bien.
pub fn open_project(db: &Db, kind: AppKind, project_id: i64) -> Result<()> {
    open_project_with(db, kind, project_id, open_path)
}

fn open_project_with(
    db: &Db,
    kind: AppKind,
    project_id: i64,
    open: impl FnOnce(AppKind, &Path, Option<&str>) -> Result<()>,
) -> Result<()> {
    let (project, settings) = db.with_conn(|conn| {
        Ok((
            projects_repo::get_by_id(conn, project_id)?,
            settings_repo::load(conn)?,
        ))
    })?;
    let preferred = match kind {
        AppKind::Ide => settings.preferred_ide,
        AppKind::Terminal => settings.preferred_terminal,
        AppKind::FileManager => String::new(),
    };

    open(kind, Path::new(&project.path), Some(&preferred))?;
    db.with_conn(|conn| projects_repo::touch_last_opened(conn, project_id, crate::core::now_ts()))
}

/// Esquemas que acepta `open_external`. Cualquier otro se rechaza.
pub const EXTERNAL_SCHEMES: &[&str] = &["http", "https"];

/// Comprueba que `raw` sea una URL web con servidor, tal y como la abriría el navegador.
pub fn validate_external_url(raw: &str) -> Result<Url> {
    let url = Url::parse(raw.trim()).map_err(|err| {
        AppError::validation("url_invalid", format!("enlace no válido: {err}"))
            .with("detail", err.to_string())
    })?;

    if !EXTERNAL_SCHEMES.contains(&url.scheme()) {
        return Err(AppError::validation(
            "url_scheme",
            format!("solo se abren enlaces http y https, no «{}:»", url.scheme()),
        )
        .with("scheme", url.scheme()));
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err(AppError::validation(
            "url_no_host",
            "el enlace no tiene servidor",
        ));
    }
    Ok(url)
}

/// Abre una URL web en el navegador predeterminado, después de validarla.
pub fn open_external(raw: &str) -> Result<()> {
    let url = validate_external_url(raw)?;
    open::that_detached(url.as_str())?;
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn external_urls_accept_http_and_https() {
        for raw in [
            "https://github.com/IzanVil/mosaic",
            "http://localhost:8080/docs",
            "HTTPS://Example.com/Ruta?q=1#ancla",
            "  https://tauri.app  ",
        ] {
            assert!(validate_external_url(raw).is_ok(), "{raw} debería valer");
        }
        assert_eq!(
            validate_external_url("HTTPS://Example.com")
                .unwrap()
                .scheme(),
            "https"
        );
    }

    #[test]
    fn external_urls_are_normalized_like_a_browser_would() {
        // El parser es el de WHATWG: lo que se valida es lo que se abre.
        let url = validate_external_url("http:/barra-sola").unwrap();
        assert_eq!(url.as_str(), "http://barra-sola/");
    }

    #[test]
    fn external_urls_reject_every_other_scheme() {
        for raw in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            "vscode://file/home/usuario",
            "mailto:alguien@example.com",
            "ftp://example.com/fichero",
            "ssh://git@github.com/IzanVil/mosaic",
        ] {
            assert!(
                matches!(validate_external_url(raw), Err(AppError::Validation(_))),
                "{raw} no debería valer"
            );
        }
    }

    #[test]
    fn external_urls_reject_relative_and_malformed_links() {
        for raw in [
            "",
            "   ",
            "docs/README.md",
            "#instalacion",
            "//example.com/sin-esquema",
            "https://",
        ] {
            assert!(
                validate_external_url(raw).is_err(),
                "{raw:?} no debería valer"
            );
        }
    }

    use std::path::PathBuf;

    use super::*;
    use crate::model::project::DiscoveredProject;

    /// Un proyecto en la base con editor y terminal preferidos, y su id.
    fn con_proyecto() -> (Db, i64) {
        let db = Db::open_in_memory().unwrap();
        let id = db
            .with_conn(|conn| {
                settings_repo::set_raw(conn, crate::config::settings::KEY_PREFERRED_IDE, "zed")?;
                settings_repo::set_raw(
                    conn,
                    crate::config::settings::KEY_PREFERRED_TERMINAL,
                    "kitty",
                )?;
                let path = PathBuf::from("/code/mosaic");
                let proyecto = DiscoveredProject {
                    name: DiscoveredProject::name_from_path(&path),
                    path,
                    is_git_repo: true,
                    primary_language: None,
                    markers: vec![".git".into()],
                };
                Ok(projects_repo::upsert_by_path(conn, &proyecto, 1_000)?.id)
            })
            .unwrap();
        (db, id)
    }

    fn abierto(db: &Db, id: i64) -> Option<i64> {
        db.with_conn(|conn| projects_repo::get_by_id(conn, id))
            .unwrap()
            .last_opened_at
    }

    #[test]
    fn opening_a_project_uses_the_preferred_application_of_its_kind() {
        let (db, id) = con_proyecto();
        for (kind, esperada) in [
            (AppKind::Ide, "zed"),
            (AppKind::Terminal, "kitty"),
            (AppKind::FileManager, ""),
        ] {
            let mut pedida = None;
            open_project_with(&db, kind, id, |k, path, preferred| {
                assert_eq!(k, kind);
                assert_eq!(path, Path::new("/code/mosaic"));
                pedida = preferred.map(str::to_string);
                Ok(())
            })
            .unwrap();
            assert_eq!(pedida.as_deref(), Some(esperada), "{kind:?}");
        }
    }

    #[test]
    fn a_successful_launch_records_when_the_project_was_opened() {
        let (db, id) = con_proyecto();
        assert_eq!(abierto(&db, id), None);
        open_project_with(&db, AppKind::Ide, id, |_, _, _| Ok(())).unwrap();
        assert!(abierto(&db, id).is_some());
    }

    #[test]
    fn a_failed_launch_does_not_count_as_opening_the_project() {
        let (db, id) = con_proyecto();
        let err = open_project_with(&db, AppKind::Ide, id, |_, _, _| {
            Err(AppError::not_found("no_editor", "ningún IDE"))
        })
        .unwrap_err();
        assert_eq!(err.code(), "no_editor");
        assert_eq!(abierto(&db, id), None);
    }

    #[test]
    fn opening_an_unknown_project_launches_nothing() {
        let (db, _) = con_proyecto();
        let err = open_project_with(&db, AppKind::Ide, 999, |_, _, _| {
            panic!("no tendría que lanzar nada")
        })
        .unwrap_err();
        assert_eq!(err.code(), "project_not_found");
    }

    #[test]
    fn detects_only_the_applications_that_are_available() {
        let apps = detect_with(AppKind::Ide, |bin| bin == "zed" || bin == "nvim");

        let ids: Vec<&str> = apps.iter().map(|app| app.id.as_str()).collect();
        assert_eq!(ids, vec!["zed", "nvim"]);
        assert_eq!(apps[0].name, "Zed");
        assert!(apps.iter().all(|app| app.kind == AppKind::Ide));
    }

    #[test]
    fn detection_keeps_the_preference_order_not_the_discovery_order() {
        // `code` va antes que `nvim` en la tabla, sea cual sea el orden de consulta.
        let apps = detect_with(AppKind::Ide, |bin| bin == "nvim" || bin == "code");

        let ids: Vec<&str> = apps.iter().map(|app| app.id.as_str()).collect();
        assert_eq!(ids, vec!["code", "nvim"]);
    }

    #[test]
    fn detects_nothing_when_no_executable_is_available() {
        assert!(detect_with(AppKind::Ide, |_| false).is_empty());
        assert!(detect_with(AppKind::Terminal, |_| false).is_empty());
    }

    #[test]
    fn the_file_manager_is_not_detected_from_a_list() {
        assert!(
            detect_with(AppKind::FileManager, |_| true).is_empty(),
            "lo resuelve el sistema, no una tabla nuestra"
        );
    }

    #[test]
    fn terminals_receive_the_directory_through_their_own_flag() {
        let kitty = TERMINALS.iter().find(|c| c.bin == "kitty").unwrap();
        let command = build_command(kitty, Path::new("/proyectos/mosaic"));

        let args: Vec<&std::ffi::OsStr> = command.get_args().collect();
        assert_eq!(args, vec!["--directory", "/proyectos/mosaic"]);
        assert_eq!(
            command.get_current_dir(),
            Some(Path::new("/proyectos/mosaic"))
        );
    }

    #[test]
    fn ides_receive_the_directory_as_a_bare_argument() {
        let code = IDES.iter().find(|c| c.bin == "code").unwrap();
        let command = build_command(code, Path::new("/proyectos/mosaic"));

        let args: Vec<&std::ffi::OsStr> = command.get_args().collect();
        assert_eq!(args, vec!["/proyectos/mosaic"]);
    }

    #[test]
    fn the_path_is_always_a_single_argument_never_a_shell_string() {
        // Una carpeta con metacaracteres de shell debe llegar intacta y entera.
        let hostil = Path::new("/proyectos/raro; rm -rf ~/ #");
        let code = IDES.iter().find(|c| c.bin == "code").unwrap();
        let command = build_command(code, hostil);

        let args: Vec<&std::ffi::OsStr> = command.get_args().collect();
        assert_eq!(args, vec![hostil.as_os_str()]);
        assert_eq!(command.get_program(), "code");
    }

    #[test]
    fn opening_a_path_that_does_not_exist_is_rejected_before_spawning() {
        let err = open_path(AppKind::Ide, Path::new("/no/existe/de/verdad"), None).unwrap_err();

        assert!(matches!(err, AppError::InvalidPath(_)));
    }

    #[test]
    fn opening_a_file_instead_of_a_directory_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("fichero.txt");
        std::fs::write(&file, "").unwrap();

        let err = open_path(AppKind::Terminal, &file, None).unwrap_err();

        assert!(matches!(err, AppError::InvalidPath(_)));
    }

    #[test]
    fn resolve_honours_the_preferred_application() {
        let chosen = resolve_with(AppKind::Ide, Some("nvim"), |bin| {
            bin == "code" || bin == "nvim"
        })
        .unwrap();

        assert_eq!(
            chosen.bin, "nvim",
            "gana la preferencia, no el orden de la tabla"
        );
    }

    #[test]
    fn resolve_falls_back_when_the_preferred_application_is_gone() {
        // El usuario eligió Zed y luego lo desinstaló.
        let chosen = resolve_with(AppKind::Ide, Some("zed"), |bin| bin == "code").unwrap();

        assert_eq!(chosen.bin, "code");
    }

    #[test]
    fn resolve_uses_the_first_available_without_a_preference() {
        for preference in [None, Some("")] {
            let chosen = resolve_with(AppKind::Ide, preference, |bin| {
                bin == "nvim" || bin == "code"
            })
            .unwrap();

            assert_eq!(
                chosen.bin, "code",
                "primera de la tabla entre las disponibles"
            );
        }
    }

    #[test]
    fn resolve_finds_nothing_when_nothing_is_installed() {
        assert!(resolve_with(AppKind::Ide, Some("code"), |_| false).is_none());
        assert!(resolve_with(AppKind::Terminal, None, |_| false).is_none());
    }

    #[test]
    fn resolve_never_returns_a_file_manager() {
        assert!(
            resolve_with(AppKind::FileManager, None, |_| true).is_none(),
            "el explorador no sale de la tabla de candidatas"
        );
    }

    #[test]
    fn is_on_path_finds_a_real_executable() {
        // `sh` existe en cualquier unix; un nombre inventado no.
        #[cfg(unix)]
        assert!(is_on_path("sh"));
        assert!(!is_on_path("ejecutable-que-no-existe-en-ningun-sitio"));
    }
}
