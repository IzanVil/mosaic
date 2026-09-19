//! Detección y lanzamiento de aplicaciones externas.
//!
//! Mosaic abre proyectos en el IDE, la terminal o el explorador de archivos del
//! usuario. Nunca se construye una línea de shell: los procesos se lanzan con
//! `Command` pasando la ruta como argumento suelto, de modo que un nombre de
//! carpeta con comillas, espacios o `;` no puede convertirse en ejecución de
//! comandos.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

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
///
/// Cada uno recibe el directorio con su propia bandera: aunque el proceso se
/// lanza con `current_dir`, varios terminales hablan con un servidor que ya
/// existe y no heredarían el directorio de trabajo.
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

/// Elige qué ejecutable usar: el preferido si sigue disponible, si no el primero
/// que se detecte.
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
    // Redundante para los terminales que aceptan bandera, pero arregla a los que
    // simplemente heredan el directorio de trabajo.
    command.current_dir(path);
    command
}

/// Abre `path` con la aplicación del tipo indicado.
///
/// `preferred` es el identificador guardado en los ajustes; si está vacío o la
/// aplicación ya no existe, se usa la primera detectada.
pub fn open_path(kind: AppKind, path: &Path, preferred: Option<&str>) -> Result<()> {
    if !path.is_dir() {
        return Err(AppError::invalid_path(path, "ya no existe en el disco"));
    }

    if kind == AppKind::FileManager {
        open::that_detached(path)?;
        tracing::info!(path = %path.display(), "abierto en el explorador de archivos");
        return Ok(());
    }

    let candidate = resolve(kind, preferred).ok_or_else(|| {
        AppError::NotFound(match kind {
            AppKind::Ide => "ningún IDE o editor reconocido en el PATH".into(),
            AppKind::Terminal => "ningún emulador de terminal reconocido en el PATH".into(),
            AppKind::FileManager => unreachable!("el explorador se resuelve antes"),
        })
    })?;

    build_command(candidate, path)
        .spawn()
        .map_err(|err| AppError::Internal(format!("no se pudo lanzar {}: {err}", candidate.bin)))?;

    tracing::info!(app = candidate.bin, path = %path.display(), "proyecto abierto");
    Ok(())
}

/// Ruta del ejecutable de una aplicación detectada, para diagnóstico.
pub fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|candidate| candidate.is_file() && is_executable(candidate))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn which_returns_the_full_path_of_an_executable() {
        #[cfg(unix)]
        {
            let path = which("sh").expect("sh debería estar en el PATH");
            assert!(path.is_absolute());
            assert!(path.ends_with("sh"));
        }
        assert_eq!(which("ejecutable-que-no-existe-en-ningun-sitio"), None);
    }
}
