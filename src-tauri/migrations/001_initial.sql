-- Migración inicial de Mosaic.
--
-- Todas las marcas de tiempo son enteros: segundos desde el epoch Unix (UTC).
-- Todos los booleanos son enteros 0/1.

-- Rutas raíz que el usuario quiere escanear.
CREATE TABLE scan_paths (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    path         TEXT NOT NULL UNIQUE,
    enabled      INTEGER NOT NULL DEFAULT 1,
    created_at   INTEGER NOT NULL,
    last_scan_at INTEGER
);

-- Proyectos detectados.
--
-- `missing` marca proyectos que siguen registrados pero ya no aparecen en disco:
-- nunca se borran automáticamente para no perder etiquetas ni notas del usuario.
-- `last_seen_at` guarda la última vez que el escáner los encontró.
CREATE TABLE projects (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    name             TEXT NOT NULL,
    path             TEXT NOT NULL UNIQUE,
    is_git_repo      INTEGER NOT NULL DEFAULT 0,
    primary_language TEXT,
    last_opened_at   INTEGER,
    pinned           INTEGER NOT NULL DEFAULT 0,
    notes            TEXT,
    missing          INTEGER NOT NULL DEFAULT 0,
    last_seen_at     INTEGER,
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL
);

-- Etiquetas.
CREATE TABLE tags (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL UNIQUE,
    color      TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

-- Relación N:M proyectos <-> etiquetas.
CREATE TABLE project_tags (
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tag_id     INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (project_id, tag_id)
);

-- Caché de estado Git (se refresca periódicamente, ver Fase 2).
CREATE TABLE git_status_cache (
    project_id      INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    branch          TEXT,
    ahead           INTEGER,
    behind          INTEGER,
    is_dirty        INTEGER,
    last_commit_sha TEXT,
    last_commit_msg TEXT,
    last_commit_at  INTEGER,
    remote_url      TEXT,
    refreshed_at    INTEGER NOT NULL
);

-- Ajustes de la aplicación (clave-valor).
CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX idx_projects_name ON projects(name);
CREATE INDEX idx_projects_last_opened ON projects(last_opened_at DESC);
CREATE INDEX idx_git_status_refreshed ON git_status_cache(refreshed_at);
