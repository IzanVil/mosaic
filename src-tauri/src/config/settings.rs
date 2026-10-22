//! Ajustes de la aplicación: claves, valores por defecto, límites y validación.
//! Se leen y guardan en `db::repositories::settings`. Toda clave tiene valor por
//! defecto, así que una base recién creada vale sin precargar nada.

use serde::{Deserialize, Serialize};

use crate::errors::{AppError, Result};

/// Profundidad máxima de escaneo, contando la raíz como nivel 0.
pub const KEY_MAX_DEPTH: &str = "scan.max_depth";
/// Directorios que el escáner nunca recorre, separados por `\n`.
pub const KEY_EXCLUDED_DIRS: &str = "scan.excluded_dirs";
/// Tope de entradas visitadas por escaneo antes de abortar con aviso.
pub const KEY_MAX_ENTRIES_PER_SCAN: &str = "scan.max_entries_per_scan";
/// Si el escaneo se lanza automáticamente al arrancar la aplicación.
pub const KEY_SCAN_ON_STARTUP: &str = "scan.on_startup";
/// Minutos entre refrescos automáticos del estado Git.
pub const KEY_GIT_REFRESH_MINUTES: &str = "git.refresh_interval_minutes";
/// Ejecutable del IDE preferido. Vacío = usar el primero que se detecte.
pub const KEY_PREFERRED_IDE: &str = "apps.preferred_ide";
/// Ejecutable del terminal preferido. Vacío = usar el primero que se detecte.
pub const KEY_PREFERRED_TERMINAL: &str = "apps.preferred_terminal";
/// La última vista del tablero en JSON: estado de la interfaz que el backend no interpreta.
pub const KEY_VIEW_STATE: &str = "ui.view_state";

/// Profundidad máxima por defecto (la raíz es el nivel 0).
pub const DEFAULT_MAX_DEPTH: usize = 4;
/// Tope de entradas por defecto.
pub const DEFAULT_MAX_ENTRIES_PER_SCAN: usize = 50_000;
/// Intervalo por defecto entre refrescos automáticos del estado Git.
pub const DEFAULT_GIT_REFRESH_MINUTES: u64 = 5;

/// Carpetas excluidas por defecto: dependencias, artefactos de build y cachés.
pub const DEFAULT_EXCLUDED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".venv",
    "venv",
    "__pycache__",
    "vendor",
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".gradle",
    ".cache",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".tox",
    "coverage",
    ".nyc_output",
    ".turbo",
    ".parcel-cache",
    ".idea",
    // No tiene marcadores nunca: excluirla siempre da lo mismo y simplifica el recorrido.
    ".vscode",
    "Pods",
    ".dart_tool",
];

/// Preferencias de la aplicación resueltas (valor guardado o valor por defecto).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub max_depth: usize,
    pub excluded_dirs: Vec<String>,
    pub max_entries_per_scan: usize,
    pub scan_on_startup: bool,
    /// Minutos entre refrescos automáticos del estado Git. `0` los desactiva.
    pub git_refresh_interval_minutes: u64,
    /// Ejecutable del IDE preferido; vacío significa "el primero disponible".
    pub preferred_ide: String,
    /// Ejecutable del terminal preferido; vacío significa "el primero disponible".
    pub preferred_terminal: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            excluded_dirs: DEFAULT_EXCLUDED_DIRS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            max_entries_per_scan: DEFAULT_MAX_ENTRIES_PER_SCAN,
            scan_on_startup: false,
            git_refresh_interval_minutes: DEFAULT_GIT_REFRESH_MINUTES,
            preferred_ide: String::new(),
            preferred_terminal: String::new(),
        }
    }
}

/// Los cinco ajustes de escaneo y Git de la pantalla de Ajustes; editor y terminal van aparte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvancedSettings {
    /// Profundidad máxima del escaneo; la raíz es el nivel 0.
    pub max_depth: usize,
    /// Nombres de carpeta que el escáner nunca recorre.
    pub excluded_dirs: Vec<String>,
    /// Entradas visitadas por escaneo antes de detenerse con aviso.
    pub max_entries_per_scan: usize,
    /// Si la aplicación escanea sola al arrancar.
    pub scan_on_startup: bool,
    /// Minutos entre refrescos automáticos de Git. `0` los desactiva.
    pub git_refresh_interval_minutes: u64,
}

impl From<&Settings> for AdvancedSettings {
    fn from(settings: &Settings) -> Self {
        Self {
            max_depth: settings.max_depth,
            excluded_dirs: settings.excluded_dirs.clone(),
            max_entries_per_scan: settings.max_entries_per_scan,
            scan_on_startup: settings.scan_on_startup,
            git_refresh_interval_minutes: settings.git_refresh_interval_minutes,
        }
    }
}

/// Límites de los ajustes avanzados; viajan al frontend para que valide con los mismos números.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsLimits {
    pub max_depth_min: usize,
    pub max_depth_max: usize,
    pub max_entries_min: usize,
    pub max_entries_max: usize,
    /// El intervalo de Git va de 0 (desactivado) a este número de minutos.
    pub git_refresh_max_minutes: u64,
    pub excluded_dirs_max: usize,
    /// Longitud máxima, en caracteres, de un nombre de carpeta excluida.
    pub excluded_dir_max_chars: usize,
}

/// Los límites vigentes; la profundidad acaba en 8: más abajo casi nunca hay proyectos.
pub const LIMITS: SettingsLimits = SettingsLimits {
    max_depth_min: 1,
    max_depth_max: 8,
    max_entries_min: 1_000,
    max_entries_max: 500_000,
    git_refresh_max_minutes: 60,
    excluded_dirs_max: 100,
    excluded_dir_max_chars: 255,
};

/// Lo que necesita la pantalla de Ajustes: valores, los de fábrica («Restaurar») y límites.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvancedSettingsView {
    pub current: AdvancedSettings,
    pub defaults: AdvancedSettings,
    pub limits: SettingsLimits,
}

/// Comprueba unos ajustes avanzados y devuelve su forma normalizada.
pub fn validate_advanced(raw: AdvancedSettings) -> Result<AdvancedSettings> {
    if !(LIMITS.max_depth_min..=LIMITS.max_depth_max).contains(&raw.max_depth) {
        return Err(AppError::validation(
            "depth_out_of_range",
            format!(
                "la profundidad tiene que estar entre {} y {}",
                LIMITS.max_depth_min, LIMITS.max_depth_max
            ),
        )
        .with("min", LIMITS.max_depth_min)
        .with("max", LIMITS.max_depth_max));
    }
    if !(LIMITS.max_entries_min..=LIMITS.max_entries_max).contains(&raw.max_entries_per_scan) {
        return Err(AppError::validation(
            "entries_out_of_range",
            format!(
                "el tope de entradas tiene que estar entre {} y {}",
                LIMITS.max_entries_min, LIMITS.max_entries_max
            ),
        )
        .with("min", LIMITS.max_entries_min)
        .with("max", LIMITS.max_entries_max));
    }
    if raw.git_refresh_interval_minutes > LIMITS.git_refresh_max_minutes {
        return Err(AppError::validation(
            "git_interval_out_of_range",
            format!(
                "el intervalo de Git tiene que estar entre 0 y {} minutos",
                LIMITS.git_refresh_max_minutes
            ),
        )
        .with("max", LIMITS.git_refresh_max_minutes));
    }
    if raw.excluded_dirs.len() > LIMITS.excluded_dirs_max {
        return Err(AppError::validation(
            "excluded_too_many",
            format!(
                "no puede haber más de {} carpetas excluidas",
                LIMITS.excluded_dirs_max
            ),
        )
        .with("max", LIMITS.excluded_dirs_max));
    }

    // Una lista vacía vale: es elegir no excluir nada, aunque se recorra `node_modules`.
    let mut excluded_dirs: Vec<String> = Vec::with_capacity(raw.excluded_dirs.len());
    for nombre in raw.excluded_dirs {
        let nombre = nombre.trim().to_string();
        if nombre.is_empty() {
            return Err(AppError::validation(
                "excluded_empty",
                "el nombre de una carpeta excluida no puede estar vacío",
            ));
        }
        if nombre.contains('/') || nombre.contains('\\') {
            return Err(AppError::validation(
                "excluded_is_path",
                format!("«{nombre}» es una ruta: se excluyen nombres de carpeta, sin barras"),
            )
            .with("name", nombre));
        }
        if nombre.chars().any(char::is_control) {
            return Err(AppError::validation(
                "excluded_control",
                "el nombre de una carpeta excluida no admite caracteres de control",
            ));
        }
        if nombre.chars().count() > LIMITS.excluded_dir_max_chars {
            return Err(AppError::validation(
                "excluded_too_long",
                format!(
                    "el nombre de una carpeta excluida no puede pasar de {} caracteres",
                    LIMITS.excluded_dir_max_chars
                ),
            )
            .with("max", LIMITS.excluded_dir_max_chars));
        }
        if excluded_dirs.contains(&nombre) {
            return Err(AppError::validation(
                "excluded_duplicate",
                format!("«{nombre}» ya está en la lista"),
            )
            .with("name", nombre));
        }
        excluded_dirs.push(nombre);
    }

    Ok(AdvancedSettings {
        excluded_dirs,
        ..raw
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn avanzados() -> AdvancedSettings {
        AdvancedSettings::from(&Settings::default())
    }

    #[test]
    fn validate_advanced_accepts_the_defaults() {
        assert_eq!(validate_advanced(avanzados()).unwrap(), avanzados());
    }

    #[test]
    fn validate_advanced_enforces_the_numeric_limits() {
        for depth in [0, 9] {
            let raw = AdvancedSettings {
                max_depth: depth,
                ..avanzados()
            };
            assert!(
                matches!(validate_advanced(raw), Err(AppError::Validation(_))),
                "{depth}"
            );
        }
        for depth in [1, 8] {
            let raw = AdvancedSettings {
                max_depth: depth,
                ..avanzados()
            };
            assert!(validate_advanced(raw).is_ok(), "{depth}");
        }
        for entries in [999, 500_001] {
            let raw = AdvancedSettings {
                max_entries_per_scan: entries,
                ..avanzados()
            };
            assert!(validate_advanced(raw).is_err(), "{entries}");
        }
        let desactivado = AdvancedSettings {
            git_refresh_interval_minutes: 0,
            ..avanzados()
        };
        assert!(
            validate_advanced(desactivado).is_ok(),
            "0 desactiva, no es un error"
        );
        let largo = AdvancedSettings {
            git_refresh_interval_minutes: 61,
            ..avanzados()
        };
        assert!(validate_advanced(largo).is_err());
    }

    #[test]
    fn validate_advanced_cleans_and_checks_excluded_dirs() {
        let con = |dirs: &[&str]| AdvancedSettings {
            excluded_dirs: dirs.iter().map(|d| (*d).to_string()).collect(),
            ..avanzados()
        };

        let limpio = validate_advanced(con(&["  node_modules ", "build"])).unwrap();
        assert_eq!(limpio.excluded_dirs, ["node_modules", "build"]);

        for malo in [
            &["   "][..],
            &["src/gen"],
            &["a\\b"],
            &["dist", "dist"],
            &["tab\tnombre"],
        ] {
            assert!(validate_advanced(con(malo)).is_err(), "{malo:?}");
        }
        assert!(validate_advanced(con(&[]))
            .unwrap()
            .excluded_dirs
            .is_empty());

        let demasiadas: Vec<String> = (0..=LIMITS.excluded_dirs_max)
            .map(|i| format!("c{i}"))
            .collect();
        let raw = AdvancedSettings {
            excluded_dirs: demasiadas,
            ..avanzados()
        };
        assert!(validate_advanced(raw).is_err());
    }
}
