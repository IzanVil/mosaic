//! Ajustes de la aplicación, persistidos en la tabla clave-valor `settings`.
//!
//! Todas las claves tienen un valor por defecto: una base de datos recién
//! creada es válida sin necesidad de precargar nada.

use rusqlite::{params, Connection, OptionalExtension};
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
/// Última vista del tablero (búsqueda, filtros, orden, sidebar) en JSON.
///
/// No forma parte de [`Settings`]: es estado de la interfaz, con una forma que
/// solo entiende el frontend, y el backend no lo interpreta más allá de
/// comprobar que es JSON válido. Se lee y escribe con [`get_raw`] y [`set_raw`]
/// desde `commands::settings`.
pub const KEY_VIEW_STATE: &str = "ui.view_state";

/// Profundidad máxima por defecto (la raíz es el nivel 0).
pub const DEFAULT_MAX_DEPTH: usize = 4;
/// Tope de entradas por defecto.
pub const DEFAULT_MAX_ENTRIES_PER_SCAN: usize = 50_000;
/// Intervalo por defecto entre refrescos automáticos del estado Git.
pub const DEFAULT_GIT_REFRESH_MINUTES: u64 = 5;

/// Directorios excluidos por defecto: dependencias, artefactos de build y
/// cachés de herramientas.
///
/// `.vscode` se excluye siempre: nunca contiene ficheros marcadores, así que
/// limitar la exclusión a "dentro de proyectos detectados" no cambiaría el
/// resultado y sí complicaría el recorrido.
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

/// Lee un valor crudo de la tabla `settings`.
pub fn get_raw(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?)
}

/// Escribe (o sobrescribe) un valor crudo en la tabla `settings`.
pub fn set_raw(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// Carga los ajustes, sustituyendo por el valor por defecto cualquier clave
/// ausente o con contenido no parseable.
pub fn load(conn: &Connection) -> Result<Settings> {
    let defaults = Settings::default();

    let max_depth = get_raw(conn, KEY_MAX_DEPTH)?
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults.max_depth);

    let excluded_dirs = match get_raw(conn, KEY_EXCLUDED_DIRS)? {
        Some(raw) => raw
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect(),
        None => defaults.excluded_dirs,
    };

    let max_entries_per_scan = get_raw(conn, KEY_MAX_ENTRIES_PER_SCAN)?
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults.max_entries_per_scan);

    let scan_on_startup = get_raw(conn, KEY_SCAN_ON_STARTUP)?
        .map(|v| v == "true" || v == "1")
        .unwrap_or(defaults.scan_on_startup);

    let git_refresh_interval_minutes = get_raw(conn, KEY_GIT_REFRESH_MINUTES)?
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults.git_refresh_interval_minutes);

    let preferred_ide = get_raw(conn, KEY_PREFERRED_IDE)?.unwrap_or(defaults.preferred_ide);
    let preferred_terminal =
        get_raw(conn, KEY_PREFERRED_TERMINAL)?.unwrap_or(defaults.preferred_terminal);

    Ok(Settings {
        max_depth,
        excluded_dirs,
        max_entries_per_scan,
        scan_on_startup,
        git_refresh_interval_minutes,
        preferred_ide,
        preferred_terminal,
    })
}

/// Los cinco ajustes de escaneo y Git que se editan desde la pantalla de
/// Ajustes. El editor y el terminal preferidos tienen sus propios comandos y
/// no van aquí.
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

/// Límites de los ajustes avanzados.
///
/// Viajan al frontend con los valores, para que la pantalla valide con los
/// mismos números que el backend en lugar de copiarlos.
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

/// Los límites vigentes. La profundidad se queda en 8: más abajo casi nunca hay
/// un proyecto, y cada nivel multiplica lo que se recorre.
pub const LIMITS: SettingsLimits = SettingsLimits {
    max_depth_min: 1,
    max_depth_max: 8,
    max_entries_min: 1_000,
    max_entries_max: 500_000,
    git_refresh_max_minutes: 60,
    excluded_dirs_max: 100,
    excluded_dir_max_chars: 255,
};

/// Lo que necesita la pantalla de Ajustes: los valores, los de fábrica (para
/// «Restaurar») y los límites.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvancedSettingsView {
    pub current: AdvancedSettings,
    pub defaults: AdvancedSettings,
    pub limits: SettingsLimits,
}

/// Comprueba unos ajustes avanzados y devuelve su forma normalizada.
///
/// Los nombres de carpeta excluida se recortan, y se rechazan los vacíos, los
/// que llevan una barra (son nombres, no rutas), los que llevan caracteres de
/// control y los repetidos. Una lista vacía es válida: es elegir no excluir
/// nada, aunque el escaneo vaya a recorrer `node_modules`.
pub fn validate_advanced(raw: AdvancedSettings) -> Result<AdvancedSettings> {
    let fuera_de_rango = |nombre: &str, min: String, max: String| {
        AppError::Validation(format!("{nombre} tiene que estar entre {min} y {max}"))
    };

    if !(LIMITS.max_depth_min..=LIMITS.max_depth_max).contains(&raw.max_depth) {
        return Err(fuera_de_rango(
            "la profundidad",
            LIMITS.max_depth_min.to_string(),
            LIMITS.max_depth_max.to_string(),
        ));
    }
    if !(LIMITS.max_entries_min..=LIMITS.max_entries_max).contains(&raw.max_entries_per_scan) {
        return Err(fuera_de_rango(
            "el tope de entradas",
            LIMITS.max_entries_min.to_string(),
            LIMITS.max_entries_max.to_string(),
        ));
    }
    if raw.git_refresh_interval_minutes > LIMITS.git_refresh_max_minutes {
        return Err(fuera_de_rango(
            "el intervalo de Git",
            "0".into(),
            format!("{} minutos", LIMITS.git_refresh_max_minutes),
        ));
    }
    if raw.excluded_dirs.len() > LIMITS.excluded_dirs_max {
        return Err(AppError::Validation(format!(
            "no puede haber más de {} carpetas excluidas",
            LIMITS.excluded_dirs_max
        )));
    }

    let mut excluded_dirs: Vec<String> = Vec::with_capacity(raw.excluded_dirs.len());
    for nombre in raw.excluded_dirs {
        let nombre = nombre.trim().to_string();
        if nombre.is_empty() {
            return Err(AppError::Validation(
                "el nombre de una carpeta excluida no puede estar vacío".into(),
            ));
        }
        if nombre.contains('/') || nombre.contains('\\') {
            return Err(AppError::Validation(format!(
                "«{nombre}» es una ruta: se excluyen nombres de carpeta, sin barras"
            )));
        }
        if nombre.chars().any(char::is_control) {
            return Err(AppError::Validation(
                "el nombre de una carpeta excluida no admite caracteres de control".into(),
            ));
        }
        if nombre.chars().count() > LIMITS.excluded_dir_max_chars {
            return Err(AppError::Validation(format!(
                "el nombre de una carpeta excluida no puede pasar de {} caracteres",
                LIMITS.excluded_dir_max_chars
            )));
        }
        if excluded_dirs.contains(&nombre) {
            return Err(AppError::Validation(format!(
                "«{nombre}» ya está en la lista"
            )));
        }
        excluded_dirs.push(nombre);
    }

    Ok(AdvancedSettings {
        excluded_dirs,
        ..raw
    })
}

/// Guarda solo los cinco ajustes avanzados; el resto de claves no se toca.
pub fn save_advanced(conn: &Connection, settings: &AdvancedSettings) -> Result<()> {
    set_raw(conn, KEY_MAX_DEPTH, &settings.max_depth.to_string())?;
    set_raw(conn, KEY_EXCLUDED_DIRS, &settings.excluded_dirs.join("\n"))?;
    set_raw(
        conn,
        KEY_MAX_ENTRIES_PER_SCAN,
        &settings.max_entries_per_scan.to_string(),
    )?;
    set_raw(
        conn,
        KEY_SCAN_ON_STARTUP,
        &settings.scan_on_startup.to_string(),
    )?;
    set_raw(
        conn,
        KEY_GIT_REFRESH_MINUTES,
        &settings.git_refresh_interval_minutes.to_string(),
    )?;
    Ok(())
}

/// Persiste todos los ajustes.
pub fn save(conn: &Connection, settings: &Settings) -> Result<()> {
    set_raw(conn, KEY_MAX_DEPTH, &settings.max_depth.to_string())?;
    set_raw(conn, KEY_EXCLUDED_DIRS, &settings.excluded_dirs.join("\n"))?;
    set_raw(
        conn,
        KEY_MAX_ENTRIES_PER_SCAN,
        &settings.max_entries_per_scan.to_string(),
    )?;
    set_raw(
        conn,
        KEY_SCAN_ON_STARTUP,
        &settings.scan_on_startup.to_string(),
    )?;
    set_raw(
        conn,
        KEY_GIT_REFRESH_MINUTES,
        &settings.git_refresh_interval_minutes.to_string(),
    )?;
    set_raw(conn, KEY_PREFERRED_IDE, &settings.preferred_ide)?;
    set_raw(conn, KEY_PREFERRED_TERMINAL, &settings.preferred_terminal)?;
    Ok(())
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

    #[test]
    fn save_advanced_round_trips_and_leaves_the_preferred_apps_alone() {
        let db = crate::db::Db::open_in_memory().unwrap();
        db.with_conn(|conn| {
            set_raw(conn, KEY_PREFERRED_IDE, "code")?;
            set_raw(conn, KEY_PREFERRED_TERMINAL, "konsole")
        })
        .unwrap();

        let nuevos = AdvancedSettings {
            max_depth: 6,
            excluded_dirs: vec![],
            max_entries_per_scan: 2_000,
            scan_on_startup: true,
            git_refresh_interval_minutes: 0,
        };
        db.with_conn(|conn| save_advanced(conn, &nuevos)).unwrap();
        let leido = db.with_conn(load).unwrap();

        assert_eq!(
            AdvancedSettings::from(&leido),
            nuevos,
            "una lista vacía se guarda vacía"
        );
        assert_eq!(leido.preferred_ide, "code");
        assert_eq!(leido.preferred_terminal, "konsole");
    }
    use crate::db::Db;

    #[test]
    fn load_returns_defaults_on_empty_database() {
        let db = Db::open_in_memory().unwrap();
        let settings = db.with_conn(load).unwrap();

        assert_eq!(settings, Settings::default());
        assert_eq!(settings.max_depth, 4);
        assert_eq!(settings.max_entries_per_scan, 50_000);
        assert!(!settings.scan_on_startup);
        assert_eq!(settings.git_refresh_interval_minutes, 5);
        assert!(settings.preferred_ide.is_empty());
        assert!(settings.preferred_terminal.is_empty());
    }

    #[test]
    fn save_then_load_roundtrips() {
        let db = Db::open_in_memory().unwrap();
        let custom = Settings {
            max_depth: 2,
            excluded_dirs: vec!["node_modules".into(), "target".into()],
            max_entries_per_scan: 100,
            scan_on_startup: true,
            git_refresh_interval_minutes: 15,
            preferred_ide: "zed".into(),
            preferred_terminal: "kitty".into(),
        };

        db.with_conn(|conn| save(conn, &custom)).unwrap();
        let loaded = db.with_conn(load).unwrap();

        assert_eq!(loaded, custom);
    }

    #[test]
    fn unparseable_values_fall_back_to_defaults() {
        let db = Db::open_in_memory().unwrap();
        db.with_conn(|conn| set_raw(conn, KEY_MAX_DEPTH, "no-es-un-numero"))
            .unwrap();

        let settings = db.with_conn(load).unwrap();
        assert_eq!(settings.max_depth, DEFAULT_MAX_DEPTH);
    }
}
