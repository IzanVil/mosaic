CREATE TABLE scan_paths (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    path         TEXT NOT NULL UNIQUE,
    enabled      INTEGER NOT NULL DEFAULT 1,
    created_at   INTEGER NOT NULL,
    last_scan_at INTEGER
);

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

CREATE TABLE tags (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL UNIQUE,
    color      TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE project_tags (
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tag_id     INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (project_id, tag_id)
);

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

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX idx_projects_name ON projects(name);
CREATE INDEX idx_projects_last_opened ON projects(last_opened_at DESC);
CREATE INDEX idx_git_status_refreshed ON git_status_cache(refreshed_at);
