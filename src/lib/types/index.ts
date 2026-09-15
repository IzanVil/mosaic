/**
 * Tipos compartidos con el backend.
 *
 * Son un espejo literal de los structs de Rust: los campos van en `snake_case`
 * porque serde los serializa con sus nombres tal cual, que a su vez coinciden
 * con las columnas SQL. Cualquier cambio aquí debe ir acompañado del cambio
 * equivalente en `src-tauri/src/`.
 */

/** Espejo de `core::project::Project`. */
export interface Project {
  id: number;
  name: string;
  /** Ruta absoluta y canónica. Es la clave natural del proyecto. */
  path: string;
  is_git_repo: boolean;
  primary_language: string | null;
  /** Segundos desde el epoch Unix. */
  last_opened_at: number | null;
  pinned: boolean;
  notes: string | null;
  /** El último escaneo de su raíz no encontró la carpeta en disco. */
  missing: boolean;
  last_seen_at: number | null;
  created_at: number;
  updated_at: number;
}

/** Espejo de `db::repositories::scan_paths::ScanPath`. */
export interface ScanPath {
  id: number;
  path: string;
  enabled: boolean;
  created_at: number;
  last_scan_at: number | null;
}

/** Espejo de `core::scanner::ScanSummary`. */
export interface ScanSummary {
  roots_scanned: number;
  roots_unavailable: number;
  entries_visited: number;
  projects_found: number;
  projects_new: number;
  projects_updated: number;
  projects_missing: number;
  /** Alguna raíz alcanzó el tope de entradas y quedó incompleta. */
  truncated: boolean;
  elapsed_ms: number;
}

/**
 * Espejo de `db::repositories::git_status::GitStatusEntry`.
 *
 * El backend serializa el `GitStatus` aplanado, así que sus campos llegan al
 * mismo nivel que `project_id` y `refreshed_at`.
 */
export interface GitStatusEntry {
  project_id: number;
  /** Cuándo se leyó el repositorio, en segundos desde el epoch Unix. */
  refreshed_at: number;
  /** Rama actual. `null` si el HEAD está separado. */
  branch: string | null;
  /** Commits por delante del upstream. `null` si la rama no tiene upstream. */
  ahead: number | null;
  /** Commits por detrás del upstream. `null` si la rama no tiene upstream. */
  behind: number | null;
  /** Hay cambios sin commitear, con el mismo criterio que `git status`. */
  is_dirty: boolean;
  last_commit_sha: string | null;
  /** Primera línea del mensaje del último commit. */
  last_commit_msg: string | null;
  last_commit_at: number | null;
  remote_url: string | null;
}

/** Espejo de `core::git::GitRefreshSummary`. */
export interface GitRefreshSummary {
  refreshed: number;
  failed: number;
  elapsed_ms: number;
}

/** Espejo de `core::launcher::AppKind`. */
export type AppKind = 'ide' | 'terminal' | 'file_manager';

/** Espejo de `core::launcher::DetectedApp`. */
export interface DetectedApp {
  /** Nombre del ejecutable; es el identificador que se guarda en los ajustes. */
  id: string;
  name: string;
  kind: AppKind;
}

/** Espejo de `commands::system::PreferredApps`. Cadena vacía = el primero disponible. */
export interface PreferredApps {
  ide: string;
  terminal: string;
}
