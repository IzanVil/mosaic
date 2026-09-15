//! Ajustes de la aplicación, persistidos en la tabla clave-valor `settings`.
//!
//! Todas las claves tienen un valor por defecto: una base de datos recién
//! creada es válida sin necesidad de precargar nada.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::errors::Result;

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
