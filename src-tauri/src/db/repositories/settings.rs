//! La tabla clave-valor `settings`. Qué claves hay, sus valores por defecto y
//! sus límites lo dice `config::settings`; aquí se leen y se guardan.

use rusqlite::{params, Connection, OptionalExtension};

use crate::config::settings::{
    AdvancedSettings, Settings, KEY_EXCLUDED_DIRS, KEY_GIT_REFRESH_MINUTES, KEY_MAX_DEPTH,
    KEY_MAX_ENTRIES_PER_SCAN, KEY_PREFERRED_IDE, KEY_PREFERRED_TERMINAL, KEY_SCAN_ON_STARTUP,
};
use crate::errors::Result;

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

/// Carga los ajustes; una clave ausente o ilegible toma su valor por defecto.
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
    save_advanced(conn, &AdvancedSettings::from(settings))?;
    set_raw(conn, KEY_PREFERRED_IDE, &settings.preferred_ide)?;
    set_raw(conn, KEY_PREFERRED_TERMINAL, &settings.preferred_terminal)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn save_advanced_round_trips_and_leaves_the_preferred_apps_alone() {
        let db = Db::open_in_memory().unwrap();
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
        assert_eq!(settings.max_depth, Settings::default().max_depth);
    }
}
